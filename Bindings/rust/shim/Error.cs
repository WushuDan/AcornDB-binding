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
        var bytes = System.Text.Encoding.UTF8.GetBytes(s + "\0");
        unsafe { fixed (byte* p = bytes) { return (IntPtr)p; } }
    }
}
