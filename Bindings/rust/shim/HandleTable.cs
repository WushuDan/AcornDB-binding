using System;
using System.Collections.Concurrent;

internal sealed class HandleTable<T>
{
    private readonly ConcurrentDictionary<ulong, T> _map = new();
    private ulong _next = 1;

    public ulong Add(T value)
    {
        var id = System.Threading.Interlocked.Increment(ref _next);
        _map[id] = value;
        return id;
    }

    public T Get(ulong id) => _map.TryGetValue(id, out var v) ? v : throw new InvalidOperationException($"Invalid handle {id}");
    public bool Remove(ulong id, out T? value) => _map.TryRemove(id, out value);
}
