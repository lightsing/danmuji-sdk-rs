using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;

namespace DanmujiSdkRust.Bridge
{
    internal static class RustNative
    {
        private const string NativeLibraryName = "danmuji_rust_plugin";
        private const string NativeDllFileName = "danmuji_rust_plugin.dll";
        private const int OverlayFooterLength = 60;
        private static readonly byte[] OverlayMagic = Encoding.ASCII.GetBytes("DMJRSOVL00000001");

        static RustNative()
        {
            string nativePath = ResolveNativeLibraryPath();
            string nativeDirectory = Path.GetDirectoryName(nativePath);

            if (!string.IsNullOrEmpty(nativeDirectory))
            {
                SetDllDirectory(nativeDirectory);
            }

            IntPtr handle = LoadLibrary(nativePath);
            if (handle == IntPtr.Zero)
            {
                throw new Win32Exception(
                    Marshal.GetLastWin32Error(),
                    "Failed to load Rust native plugin: " + nativePath);
            }
        }

        [DllImport("kernel32", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern bool SetDllDirectory(string lpPathName);

        [DllImport("kernel32", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern IntPtr LoadLibrary(string lpFileName);

        private static string ResolveNativeLibraryPath()
        {
            string assemblyPath = Assembly.GetExecutingAssembly().Location;
            string overlayPath = TryExtractOverlayNativeDll(assemblyPath);

            if (!string.IsNullOrEmpty(overlayPath))
            {
                return overlayPath;
            }

            string assemblyDirectory = Path.GetDirectoryName(assemblyPath);
            if (string.IsNullOrEmpty(assemblyDirectory))
            {
                assemblyDirectory = AppDomain.CurrentDomain.BaseDirectory;
            }

            return Path.Combine(assemblyDirectory, NativeDllFileName);
        }

        private static string TryExtractOverlayNativeDll(string assemblyPath)
        {
            if (string.IsNullOrEmpty(assemblyPath) || !File.Exists(assemblyPath))
            {
                return null;
            }

            using (FileStream stream = new FileStream(
                assemblyPath,
                FileMode.Open,
                FileAccess.Read,
                FileShare.ReadWrite | FileShare.Delete))
            {
                if (stream.Length < OverlayFooterLength)
                {
                    return null;
                }

                byte[] footer = new byte[OverlayFooterLength];
                stream.Seek(-OverlayFooterLength, SeekOrigin.End);
                ReadExactly(stream, footer, 0, footer.Length);

                if (!FooterHasMagic(footer))
                {
                    return null;
                }

                uint version = BitConverter.ToUInt32(footer, 40);
                if (version != 1)
                {
                    throw new InvalidOperationException("Unsupported danmuji native overlay version: " + version);
                }

                ulong payloadLength = BitConverter.ToUInt64(footer, 32);
                if (payloadLength > int.MaxValue || payloadLength > (ulong)(stream.Length - OverlayFooterLength))
                {
                    throw new InvalidOperationException("Invalid danmuji native overlay length.");
                }

                long payloadOffset = stream.Length - OverlayFooterLength - (long)payloadLength;
                byte[] payload = new byte[(int)payloadLength];
                stream.Seek(payloadOffset, SeekOrigin.Begin);
                ReadExactly(stream, payload, 0, payload.Length);

                byte[] expectedHash = new byte[32];
                Buffer.BlockCopy(footer, 0, expectedHash, 0, expectedHash.Length);
                byte[] actualHash;
                using (SHA256 sha256 = SHA256.Create())
                {
                    actualHash = sha256.ComputeHash(payload);
                }

                if (!BytesEqual(expectedHash, actualHash))
                {
                    throw new InvalidOperationException("Rust native overlay hash mismatch.");
                }

                string hashText = ToHex(actualHash);
                string directory = Path.Combine(Path.GetTempPath(), "danmuji-sdk-rs", hashText);
                Directory.CreateDirectory(directory);

                string nativePath = Path.Combine(directory, NativeDllFileName);
                if (!File.Exists(nativePath) || new FileInfo(nativePath).Length != payload.Length)
                {
                    File.WriteAllBytes(nativePath, payload);
                }

                return nativePath;
            }
        }

        private static bool FooterHasMagic(byte[] footer)
        {
            int offset = 44;
            for (int i = 0; i < OverlayMagic.Length; i++)
            {
                if (footer[offset + i] != OverlayMagic[i])
                {
                    return false;
                }
            }

            return true;
        }

        private static void ReadExactly(Stream stream, byte[] buffer, int offset, int count)
        {
            int readTotal = 0;
            while (readTotal < count)
            {
                int read = stream.Read(buffer, offset + readTotal, count - readTotal);
                if (read == 0)
                {
                    throw new EndOfStreamException();
                }

                readTotal += read;
            }
        }

        private static bool BytesEqual(byte[] left, byte[] right)
        {
            if (left.Length != right.Length)
            {
                return false;
            }

            int diff = 0;
            for (int i = 0; i < left.Length; i++)
            {
                diff |= left[i] ^ right[i];
            }

            return diff == 0;
        }

        private static string ToHex(byte[] bytes)
        {
            char[] chars = new char[bytes.Length * 2];
            const string hex = "0123456789abcdef";

            for (int i = 0; i < bytes.Length; i++)
            {
                chars[i * 2] = hex[bytes[i] >> 4];
                chars[i * 2 + 1] = hex[bytes[i] & 0x0f];
            }

            return new string(chars);
        }

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_metadata")]
        internal static extern void PluginMetadata(out RustPluginMetadata metadata);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_set_host")]
        internal static extern void SetHost(ref RustHostApi api);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_inited")]
        internal static extern void Inited(RustPluginContext context);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_start")]
        internal static extern void Start(RustPluginContext context);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_stop")]
        internal static extern void Stop(RustPluginContext context);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_admin")]
        internal static extern void Admin(RustPluginContext context);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_deinit")]
        internal static extern void DeInit(RustPluginContext context);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_on_connected")]
        internal static extern void OnConnected(int roomId);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_on_disconnected")]
        internal static extern void OnDisconnected(RustFfiStr error);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_on_room_count")]
        internal static extern void OnRoomCount(uint userCount);

        [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "danmuji_rs_plugin_on_danmaku")]
        internal static extern void OnDanmaku(ref RustDanmaku danmaku);

        internal static string ToManagedString(RustFfiStr value)
        {
            if (value.Ptr == IntPtr.Zero)
            {
                return string.Empty;
            }

            ulong rawLength = value.Len.ToUInt64();
            if (rawLength > int.MaxValue)
            {
                throw new InvalidOperationException("Native string is too large.");
            }

            int length = (int)rawLength;
            byte[] bytes = new byte[length];
            Marshal.Copy(value.Ptr, bytes, 0, length);
            return Encoding.UTF8.GetString(bytes);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RustFfiStr
    {
        public IntPtr Ptr;
        public UIntPtr Len;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RustPluginMetadata
    {
        public RustFfiStr Name;
        public RustFfiStr Author;
        public RustFfiStr Contact;
        public RustFfiStr Version;
        public RustFfiStr Description;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RustPluginContext
    {
        public int HasRoomId;
        public int RoomId;
        public int Status;
        public int DebugMode;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RustHostApi
    {
        public IntPtr UserData;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostLogDelegate Log;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostAddDmDelegate AddDm;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostSendSspMsgDelegate SendSspMsg;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostGetRoomIdDelegate GetRoomId;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostGetFlagDelegate GetStatus;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostGetFlagDelegate GetDebugMode;

        [MarshalAs(UnmanagedType.FunctionPtr)]
        public HostGetStringDelegate GetPluginPath;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RustGiftRank
    {
        public RustFfiStr UserName;
        public RustFfiStr Coin;
        public int Uid;
        public long UidLong;
        public RustFfiStr UidStr;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RustDanmaku
    {
        public int MsgType;
        public int InteractType;
        public RustFfiStr CommentText;
        public RustFfiStr CommentUser;
        public RustFfiStr UserName;
        public int UserId;
        public long UserIdLong;
        public RustFfiStr UserIdStr;
        public int UserGuardLevel;
        public RustFfiStr GiftUser;
        public RustFfiStr GiftName;
        public RustFfiStr GiftNum;
        public int GiftCount;
        public RustFfiStr GiftRcost;
        public IntPtr GiftRankingPtr;
        public UIntPtr GiftRankingLen;
        public int IsAdmin;
        public int IsVip;
        public RustFfiStr RoomId;
        public RustFfiStr RawData;
        public int JsonVersion;
        public RustFfiStr Price;
        public int ScKeepTime;
        public long WatchedCount;
        public RustFfiStr RawDataJToken;
    }

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void HostLogDelegate(IntPtr userdata, RustFfiStr text);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void HostAddDmDelegate(IntPtr userdata, RustFfiStr text, int fullscreen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void HostSendSspMsgDelegate(IntPtr userdata, RustFfiStr text);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate int HostGetRoomIdDelegate(IntPtr userdata, out int roomId);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate int HostGetFlagDelegate(IntPtr userdata);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate UIntPtr HostGetStringDelegate(IntPtr userdata, IntPtr buffer, UIntPtr bufferLen);
}
