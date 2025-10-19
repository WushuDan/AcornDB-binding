using System;
using System.Runtime.InteropServices;

public static class NativeExports
{
    static readonly HandleTable<AcornFacade.JsonTree> Trees = new();
    static readonly HandleTable<AcornFacade.JsonIterator> Iterators = new();

    [UnmanagedCallersOnly(EntryPoint = "acorn_open_tree")]
    public static int OpenTree(IntPtr uriUtf8, IntPtr handlePtr)
    {
        try {
            string uri = Utf8.In(uriUtf8);
            var tree = AcornFacade.OpenJsonTree(uri);
            ulong handle = Trees.Add(tree);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
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
    public static unsafe int Crack(ulong handle, IntPtr idUtf8, IntPtr outBufPtr)
    {
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
                var buf = new AcornBuf { data = mem, len = (nuint)bytes.Length };
                *(AcornBuf*)outBufPtr = buf;
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
    public static int Count(ulong handle, IntPtr countPtr)
    {
        try {
            var tree = Trees.Get(handle);
            nuint count = (nuint)tree.Count();
            unsafe { *(nuint*)countPtr = count; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(nuint*)countPtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_free_buf")]
    public static unsafe void FreeBuf(IntPtr bufPtr)
    {
        var buf = *(AcornBuf*)bufPtr;
        if (buf.data != null) {
            NativeMemory.Free(buf.data);
            buf.data = null;
            buf.len = 0;
            *(AcornBuf*)bufPtr = buf;
        }
    }

    // Iterator exports

    [UnmanagedCallersOnly(EntryPoint = "acorn_iter_start")]
    public static int IterStart(ulong treeHandle, IntPtr prefixUtf8, IntPtr outIterPtr)
    {
        try {
            var tree = Trees.Get(treeHandle);
            string prefix = Utf8.In(prefixUtf8);
            var iterator = tree.CreateIterator(prefix);
            ulong iterHandle = Iterators.Add(iterator);
            unsafe { *(ulong*)outIterPtr = iterHandle; }
            return 0;
        } catch (Exception ex) {
            unsafe { *(ulong*)outIterPtr = 0; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_iter_next")]
    public static unsafe int IterNext(ulong iterHandle, IntPtr outKeyPtr, IntPtr outJsonPtr, IntPtr outDonePtr)
    {
        try {
            var iterator = Iterators.Get(iterHandle);

            bool hasItem = iterator.Next(out string key, out byte[] json);

            // Set done flag
            *(int*)outDonePtr = hasItem ? 0 : 1;

            if (hasItem)
            {
                // Allocate and copy key
                var keyBytes = System.Text.Encoding.UTF8.GetBytes(key);
                var keyMem = (byte*)NativeMemory.Alloc((nuint)keyBytes.Length);
                if (keyMem == null) {
                    Error.Set("Failed to allocate memory for key");
                    return -1;
                }
                fixed (byte* p = keyBytes) {
                    new ReadOnlySpan<byte>(p, keyBytes.Length).CopyTo(new Span<byte>(keyMem, keyBytes.Length));
                }
                var keyBuf = new AcornBuf { data = keyMem, len = (nuint)keyBytes.Length };
                *(AcornBuf*)outKeyPtr = keyBuf;

                // Allocate and copy JSON
                var jsonMem = (byte*)NativeMemory.Alloc((nuint)json.Length);
                if (jsonMem == null) {
                    NativeMemory.Free(keyMem); // Clean up key memory
                    Error.Set("Failed to allocate memory for JSON");
                    return -1;
                }
                fixed (byte* p = json) {
                    new ReadOnlySpan<byte>(p, json.Length).CopyTo(new Span<byte>(jsonMem, json.Length));
                }
                var jsonBuf = new AcornBuf { data = jsonMem, len = (nuint)json.Length };
                *(AcornBuf*)outJsonPtr = jsonBuf;
            }
            else
            {
                // No more items - set empty buffers
                *(AcornBuf*)outKeyPtr = new AcornBuf { data = null, len = 0 };
                *(AcornBuf*)outJsonPtr = new AcornBuf { data = null, len = 0 };
            }

            return 0;
        } catch (Exception ex) {
            unsafe { *(int*)outDonePtr = 1; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_iter_close")]
    public static int IterClose(ulong iterHandle)
    {
        try {
            if (Iterators.Remove(iterHandle, out var iterator))
            {
                iterator?.Dispose();
            }
            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }
}
