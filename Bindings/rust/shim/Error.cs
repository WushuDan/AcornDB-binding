using System;
using System.Runtime.InteropServices;

internal static class Error
{
    [ThreadStatic]
    private static string? _last;

    public static void Set(Exception ex) => _last = ex.ToString();
    public static void Set(string message) => _last = message;

    [UnmanagedCallersOnly(EntryPoint = "acorn_error_message")]
    public static IntPtr Last()
    {
        var s = _last ?? string.Empty;
        // Allocate unmanaged memory that will be freed by the caller
        var bytes = System.Text.Encoding.UTF8.GetBytes(s + "\0");
        var ptr = Marshal.AllocHGlobal(bytes.Length);
        Marshal.Copy(bytes, 0, ptr, bytes.Length);
        return ptr;
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_free_error_string")]
    public static void FreeErrorString(IntPtr str)
    {
        if (str != IntPtr.Zero)
        {
            Marshal.FreeHGlobal(str);
        }
    }
}
