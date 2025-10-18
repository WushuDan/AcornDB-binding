using System;
using System.Runtime.InteropServices;

public static class NativeExports
{
    static readonly HandleTable<AcornFacade.JsonTree> Trees = new();

    [UnmanagedCallersOnly(EntryPoint = "acorn_open_tree")]
    public static int OpenTree(IntPtr uriUtf8, out ulong handle)
    {
        try {
            string uri = Utf8.In(uriUtf8);
            var tree = AcornFacade.OpenJsonTree(uri);
            handle = Trees.Add(tree);
            return 0;
        } catch (Exception ex) { handle = 0; Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_close_tree")]
    public static int CloseTree(ulong handle)
    {
        try {
            Trees.Remove(handle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_stash_json")]
    public static unsafe int Stash(ulong handle, IntPtr idUtf8, IntPtr data, nuint len)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            var span = new ReadOnlySpan<byte>(data.ToPointer(), checked((int)len));
            tree.Stash(id, span);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct AcornBuf { public unsafe byte* data; public nuint len; }

    [UnmanagedCallersOnly(EntryPoint = "acorn_crack_json")]
    public static unsafe int Crack(ulong handle, IntPtr idUtf8, out AcornBuf outBuf)
    {
        outBuf = default;
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            var bytes = tree.Crack(id);
            if (bytes is null) return 1; // not found
            fixed (byte* p = bytes) {
                var mem = (byte*)NativeMemory.Alloc((nuint)bytes.Length);
                new ReadOnlySpan<byte>(p, bytes.Length).CopyTo(new Span<byte>(mem, bytes.Length));
                outBuf = new AcornBuf { data = mem, len = (nuint)bytes.Length };
            }
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_free_buf")]
    public static unsafe void FreeBuf(ref AcornBuf buf)
    {
        if (buf.data != null) { NativeMemory.Free(buf.data); buf.data = null; buf.len = 0; }
    }
}
