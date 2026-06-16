using System;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using BilibiliDM_PluginFramework;
using Newtonsoft.Json;

namespace DanmujiSdkRust.Bridge
{
    public sealed class RustPlugin : DMPlugin
    {
        private static readonly HostLogDelegate LogCallback = HostLog;
        private static readonly HostAddDmDelegate AddDmCallback = HostAddDm;
        private static readonly HostSendSspMsgDelegate SendSspMsgCallback = HostSendSspMsg;
        private static readonly HostGetRoomIdDelegate GetRoomIdCallback = HostGetRoomId;
        private static readonly HostGetFlagDelegate GetStatusCallback = HostGetStatus;
        private static readonly HostGetFlagDelegate GetDebugModeCallback = HostGetDebugMode;
        private static readonly HostGetStringDelegate GetPluginPathCallback = HostGetPluginPath;

        private readonly GCHandle selfHandle;
        private readonly string pluginPath;
        private bool nativeAvailable;

        public RustPlugin()
        {
            nativeAvailable = true;
            selfHandle = GCHandle.Alloc(this);
            pluginPath = Assembly.GetExecutingAssembly().Location;

            PluginName = "Rust Plugin";
            PluginAuth = "";
            PluginCont = "";
            PluginVer = "";
            PluginDesc = "Rust plugin bridge";

            Connected += HandleConnected;
            Disconnected += HandleDisconnected;
            ReceivedDanmaku += HandleReceivedDanmaku;
            ReceivedRoomCount += HandleReceivedRoomCount;

            TryInitializeNative();
        }

        public override void Inited()
        {
            base.Inited();
            TryCall(() => RustNative.Inited(CreateContext()));
        }

        public override void Start()
        {
            base.Start();
            TryCall(() => RustNative.Start(CreateContext()));
        }

        public override void Stop()
        {
            base.Stop();
            TryCall(() => RustNative.Stop(CreateContext()));
        }

        public override void Admin()
        {
            base.Admin();
            TryCall(() => RustNative.Admin(CreateContext()));
        }

        public override void DeInit()
        {
            base.DeInit();
            TryCall(() => RustNative.DeInit(CreateContext()));
        }

        private void TryInitializeNative()
        {
            TryCall(() =>
            {
                RustPluginMetadata metadata;
                RustNative.PluginMetadata(out metadata);

                PluginName = RustNative.ToManagedString(metadata.Name);
                PluginAuth = RustNative.ToManagedString(metadata.Author);
                PluginCont = RustNative.ToManagedString(metadata.Contact);
                PluginVer = RustNative.ToManagedString(metadata.Version);
                PluginDesc = RustNative.ToManagedString(metadata.Description);

                RustHostApi api = new RustHostApi
                {
                    UserData = GCHandle.ToIntPtr(selfHandle),
                    Log = LogCallback,
                    AddDm = AddDmCallback,
                    SendSspMsg = SendSspMsgCallback,
                    GetRoomId = GetRoomIdCallback,
                    GetStatus = GetStatusCallback,
                    GetDebugMode = GetDebugModeCallback,
                    GetPluginPath = GetPluginPathCallback
                };

                RustNative.SetHost(ref api);
            });
        }

        private RustPluginContext CreateContext()
        {
            int roomId = 0;
            bool hasRoomId = RoomId.HasValue;

            if (hasRoomId)
            {
                roomId = RoomId.Value;
            }

            return new RustPluginContext
            {
                HasRoomId = hasRoomId ? 1 : 0,
                RoomId = roomId,
                Status = Status ? 1 : 0,
                DebugMode = DebugMode ? 1 : 0
            };
        }

        private void TryCall(Action action)
        {
            if (!nativeAvailable)
            {
                return;
            }

            try
            {
                action();
            }
            catch (Exception ex)
            {
                nativeAvailable = false;
                PluginName = "Rust Plugin Load Failed";
                PluginDesc = ex.GetType().Name + ": " + ex.Message;
            }
        }

        private void HandleConnected(object sender, ConnectedEvtArgs e)
        {
            TryCall(() => RustNative.OnConnected(e == null ? 0 : e.roomid));
        }

        private void HandleDisconnected(object sender, DisconnectEvtArgs e)
        {
            TryCall(() =>
            {
                using (FfiScope scope = new FfiScope())
                {
                    string error = e == null || e.Error == null ? null : e.Error.ToString();
                    RustNative.OnDisconnected(scope.String(error));
                }
            });
        }

        private void HandleReceivedRoomCount(object sender, ReceivedRoomCountArgs e)
        {
            TryCall(() => RustNative.OnRoomCount(e == null ? 0 : e.UserCount));
        }

        private void HandleReceivedDanmaku(object sender, ReceivedDanmakuArgs e)
        {
            if (e == null || e.Danmaku == null)
            {
                return;
            }

            TryCall(() =>
            {
                using (FfiScope scope = new FfiScope())
                {
                    RustDanmaku danmaku = ToRustDanmaku(e.Danmaku, scope);
                    RustNative.OnDanmaku(ref danmaku);
                }
            });
        }

        private static RustDanmaku ToRustDanmaku(DanmakuModel model, FfiScope scope)
        {
            RustGiftRank[] giftRanks = model.GiftRanking == null
                ? new RustGiftRank[0]
                : model.GiftRanking.Select(rank => new RustGiftRank
                {
                    UserName = scope.String(rank.UserName),
                    Coin = scope.String(rank.coin.ToString(CultureInfo.InvariantCulture)),
                    Uid = ToLegacyInt(rank.uid_long),
                    UidLong = rank.uid_long,
                    UidStr = scope.String(rank.uid_str)
                }).ToArray();

            return new RustDanmaku
            {
                MsgType = (int)model.MsgType,
                InteractType = (int)model.InteractType,
                CommentText = scope.String(model.CommentText),
                CommentUser = scope.String(model.UserName),
                UserName = scope.String(model.UserName),
                UserId = ToLegacyInt(model.UserID_long),
                UserIdLong = model.UserID_long,
                UserIdStr = scope.String(model.UserID_str),
                UserGuardLevel = model.UserGuardLevel,
                GiftUser = scope.String(model.UserName),
                GiftName = scope.String(model.GiftName),
                GiftNum = scope.String(model.GiftCount.ToString(CultureInfo.InvariantCulture)),
                GiftCount = model.GiftCount,
                GiftRcost = scope.String(null),
                GiftRankingPtr = scope.Array(giftRanks),
                GiftRankingLen = new UIntPtr((uint)giftRanks.Length),
                IsAdmin = model.isAdmin ? 1 : 0,
                IsVip = model.isVIP ? 1 : 0,
                RoomId = scope.String(model.roomID),
                RawData = scope.String(null),
                JsonVersion = model.JSON_Version,
                Price = scope.String(model.Price.ToString(CultureInfo.InvariantCulture)),
                ScKeepTime = model.SCKeepTime,
                WatchedCount = model.WatchedCount,
                RawDataJToken = scope.String(model.RawDataJToken == null ? null : model.RawDataJToken.ToString(Formatting.None))
            };
        }

        private static int ToLegacyInt(long value)
        {
            if (value < int.MinValue || value > int.MaxValue)
            {
                return -1;
            }

            return (int)value;
        }

        private static bool TryGetPlugin(IntPtr userdata, out RustPlugin plugin)
        {
            plugin = null;

            if (userdata == IntPtr.Zero)
            {
                return false;
            }

            try
            {
                GCHandle handle = GCHandle.FromIntPtr(userdata);
                plugin = handle.Target as RustPlugin;
                return plugin != null;
            }
            catch
            {
                return false;
            }
        }

        private static void HostLog(IntPtr userdata, RustFfiStr text)
        {
            RustPlugin plugin;
            if (TryGetPlugin(userdata, out plugin))
            {
                plugin.Log(RustNative.ToManagedString(text));
            }
        }

        private static void HostAddDm(IntPtr userdata, RustFfiStr text, int fullscreen)
        {
            RustPlugin plugin;
            if (TryGetPlugin(userdata, out plugin))
            {
                plugin.AddDM(RustNative.ToManagedString(text), fullscreen != 0);
            }
        }

        private static void HostSendSspMsg(IntPtr userdata, RustFfiStr text)
        {
            RustPlugin plugin;
            if (TryGetPlugin(userdata, out plugin))
            {
                plugin.SendSSPMsg(RustNative.ToManagedString(text));
            }
        }

        private static int HostGetRoomId(IntPtr userdata, out int roomId)
        {
            roomId = 0;

            RustPlugin plugin;
            if (!TryGetPlugin(userdata, out plugin) || !plugin.RoomId.HasValue)
            {
                return 0;
            }

            roomId = plugin.RoomId.Value;
            return 1;
        }

        private static int HostGetStatus(IntPtr userdata)
        {
            RustPlugin plugin;
            return TryGetPlugin(userdata, out plugin) && plugin.Status ? 1 : 0;
        }

        private static int HostGetDebugMode(IntPtr userdata)
        {
            RustPlugin plugin;
            return TryGetPlugin(userdata, out plugin) && plugin.DebugMode ? 1 : 0;
        }

        private static UIntPtr HostGetPluginPath(IntPtr userdata, IntPtr buffer, UIntPtr bufferLen)
        {
            RustPlugin plugin;
            if (!TryGetPlugin(userdata, out plugin) || string.IsNullOrEmpty(plugin.pluginPath))
            {
                return UIntPtr.Zero;
            }

            byte[] bytes = Encoding.UTF8.GetBytes(plugin.pluginPath);
            ulong capacity = bufferLen.ToUInt64();
            if (buffer != IntPtr.Zero && capacity >= (ulong)bytes.Length)
            {
                Marshal.Copy(bytes, 0, buffer, bytes.Length);
            }

            return new UIntPtr((uint)bytes.Length);
        }
    }
}
