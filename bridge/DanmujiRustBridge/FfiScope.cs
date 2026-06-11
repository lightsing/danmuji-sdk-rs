using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

namespace DanmujiSdkRust.Bridge
{
    internal sealed class FfiScope : IDisposable
    {
        private readonly List<GCHandle> handles = new List<GCHandle>();

        public RustFfiStr String(string value)
        {
            if (value == null)
            {
                return default(RustFfiStr);
            }

            byte[] bytes = Encoding.UTF8.GetBytes(value);
            int len = bytes.Length;

            if (bytes.Length == 0)
            {
                bytes = new byte[1];
            }

            GCHandle handle = GCHandle.Alloc(bytes, GCHandleType.Pinned);
            handles.Add(handle);

            return new RustFfiStr
            {
                Ptr = handle.AddrOfPinnedObject(),
                Len = new UIntPtr((uint)len)
            };
        }

        public IntPtr Array<T>(T[] values) where T : struct
        {
            if (values == null || values.Length == 0)
            {
                return IntPtr.Zero;
            }

            GCHandle handle = GCHandle.Alloc(values, GCHandleType.Pinned);
            handles.Add(handle);
            return handle.AddrOfPinnedObject();
        }

        public void Dispose()
        {
            for (int i = handles.Count - 1; i >= 0; i--)
            {
                handles[i].Free();
            }

            handles.Clear();
        }
    }
}

