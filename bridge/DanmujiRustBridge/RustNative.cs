using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Reflection;
using System.Text;

namespace DanmujiSdkRust.Bridge
{
    internal static class RustNative
    {
        private const string NativeLibraryName = "danmuji_rust_plugin";

        static RustNative()
        {
            string assemblyPath = Assembly.GetExecutingAssembly().Location;
            string assemblyDirectory = Path.GetDirectoryName(assemblyPath);

            if (!string.IsNullOrEmpty(assemblyDirectory))
            {
                SetDllDirectory(assemblyDirectory);
            }
        }

        [DllImport("kernel32", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern bool SetDllDirectory(string lpPathName);

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
}

