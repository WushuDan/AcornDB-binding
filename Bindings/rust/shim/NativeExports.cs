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
            
            // Allocate unmanaged memory and copy the data
            var mem = (byte*)NativeMemory.Alloc((nuint)bytes.Length);
            if (mem == null) {
                Error.Set("Failed to allocate memory for JSON data");
                return -1;
            }
            
            try {
                fixed (byte* p = bytes) {
                    new ReadOnlySpan<byte>(p, bytes.Length).CopyTo(new Span<byte>(mem, bytes.Length));
                }
                outBuf = new AcornBuf { data = mem, len = (nuint)bytes.Length };
                return 0;
            } catch {
                // If copying fails, free the allocated memory
                NativeMemory.Free(mem);
                throw;
            }
        } catch (Exception ex) { 
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_delete")]
    public static int Delete(ulong handle, IntPtr idUtf8)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            tree.Delete(id);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_exists")]
    public static int Exists(ulong handle, IntPtr idUtf8)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            return tree.Exists(id) ? 1 : 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_count")]
    public static int Count(ulong handle, out nuint count)
    {
        count = 0;
        try {
            var tree = Trees.Get(handle);
            count = (nuint)tree.Count();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_free_buf")]
    public static unsafe void FreeBuf(ref AcornBuf buf)
    {
        if (buf.data != null) { NativeMemory.Free(buf.data); buf.data = null; buf.len = 0; }
    }
}
