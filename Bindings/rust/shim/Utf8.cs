using System;
using System.Runtime.InteropServices;

internal static class Utf8
{
    public static unsafe string In(IntPtr p)
    {
        if (p == IntPtr.Zero) return string.Empty;
        int len = 0;
        byte* b = (byte*)p;
        while (b[len] != 0) len++;
        return System.Text.Encoding.UTF8.GetString(b, len);
    }

    public static unsafe void Out(ReadOnlySpan<byte> bytes, out IntPtr ptr, out nuint len)
    {
        var mem = (byte*)NativeMemory.Alloc((nuint)bytes.Length + 1);
        bytes.CopyTo(new Span<byte>(mem, bytes.Length));
        mem[bytes.Length] = 0;
        ptr = (IntPtr)mem;
        len = (nuint)bytes.Length;
    }
}
