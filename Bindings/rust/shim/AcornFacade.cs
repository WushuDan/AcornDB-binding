using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using AcornDB;
using AcornDB.Transaction;
using AcornDB.Storage;
using AcornDB.Shim;
using AcornDB.Reactive;

internal static class AcornFacade
{
    // Real AcornDB integration using Tree<object> for JSON storage
    // This avoids JsonElement serialization issues with Newtonsoft.Json
    internal sealed class JsonTree
    {
        private readonly Tree<object> _tree;

        public JsonTree(Tree<object> tree)
        {
            _tree = tree;
        }

        public void Stash(string id, ReadOnlySpan<byte> json)
        {
            try
            {
                // Parse JSON bytes into object using System.Text.Json with source generation
                var jsonString = System.Text.Encoding.UTF8.GetString(json);
                var obj = System.Text.Json.JsonSerializer.Deserialize(jsonString, JsonContext.Default.Object);
                if (obj != null)
                {
                    _tree.Stash(id, obj);
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to stash JSON for id '{id}': {ex.Message}", ex);
            }
        }

        public byte[]? Crack(string id)
        {
            try
            {
                var obj = _tree.Crack(id);
                if (obj == null) return null;

                // Serialize object back to bytes using System.Text.Json with source generation
                var jsonString = System.Text.Json.JsonSerializer.Serialize(obj, JsonContext.Default.Object);
                return System.Text.Encoding.UTF8.GetBytes(jsonString);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to crack JSON for id '{id}': {ex.Message}", ex);
            }
        }

        public void Delete(string id)
        {
            try
            {
                _tree.Toss(id);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to delete item with id '{id}': {ex.Message}", ex);
            }
        }

        public bool Exists(string id)
        {
            try
            {
                var obj = _tree.Crack(id);
                return obj != null;
            }
            catch
            {
                return false;
            }
        }

        public int Count()
        {
            try
            {
                return _tree.NutCount;
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to count items: {ex.Message}", ex);
            }
        }

        public JsonIterator CreateIterator(string prefix)
        {
            return new JsonIterator(_tree, prefix);
        }

        public JsonSubscription Subscribe(Action<string, byte[]> callback)
        {
            return new JsonSubscription(_tree, callback);
        }

        public async Task SyncHttpAsync(string url)
        {
            try
            {
                // Create a Branch for HTTP synchronization
                var branch = new AcornDB.Sync.Branch(url, AcornDB.Sync.SyncMode.Bidirectional);

                // ShakeAsync pulls data from the remote
                await branch.ShakeAsync(_tree);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to sync with '{url}': {ex.Message}", ex);
            }
        }

        public void BatchStash(string[] ids, byte[][] jsons)
        {
            try
            {
                if (ids.Length != jsons.Length)
                {
                    throw new ArgumentException("ids and jsons arrays must have the same length");
                }

                for (int i = 0; i < ids.Length; i++)
                {
                    var jsonString = System.Text.Encoding.UTF8.GetString(jsons[i]);
                    var obj = System.Text.Json.JsonSerializer.Deserialize(jsonString, JsonContext.Default.Object);
                    if (obj != null)
                    {
                        _tree.Stash(ids[i], obj);
                    }
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to batch stash: {ex.Message}", ex);
            }
        }

        public byte[]?[] BatchCrack(string[] ids)
        {
            try
            {
                var results = new byte[]?[ids.Length];

                for (int i = 0; i < ids.Length; i++)
                {
                    var obj = _tree.Crack(ids[i]);
                    if (obj == null)
                    {
                        results[i] = null;
                    }
                    else
                    {
                        var jsonString = System.Text.Json.JsonSerializer.Serialize(obj, JsonContext.Default.Object);
                        results[i] = System.Text.Encoding.UTF8.GetBytes(jsonString);
                    }
                }

                return results;
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to batch crack: {ex.Message}", ex);
            }
        }

        public void BatchDelete(string[] ids)
        {
            try
            {
                foreach (var id in ids)
                {
                    _tree.Toss(id);
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to batch delete: {ex.Message}", ex);
            }
        }

        public JsonTransaction BeginTransaction()
        {
            try
            {
                var transaction = _tree.BeginTransaction();
                return new JsonTransaction(transaction);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to begin transaction: {ex.Message}", ex);
            }
        }
    }

    /// <summary>
    /// Iterator over key-value pairs in a tree, filtered by prefix.
    /// Holds a snapshot of matching entries at creation time.
    /// </summary>
    internal sealed class JsonIterator
    {
        private readonly IEnumerator<Nut<object>> _enumerator;
        private bool _done;

        public JsonIterator(Tree<object> tree, string prefix)
        {
            // Get all nuts and filter by prefix (case-sensitive)
            var filteredNuts = tree.GetAllNuts()
                .Where(nut => string.IsNullOrEmpty(prefix) || nut.Id.StartsWith(prefix))
                .OrderBy(nut => nut.Id) // Ensure consistent ordering
                .ToList(); // Snapshot at this point in time

            _enumerator = filteredNuts.GetEnumerator();
            _done = false;
        }

        /// <summary>
        /// Advance to the next item. Returns true if there's a current item, false if done.
        /// </summary>
        public bool Next(out string key, out byte[] json)
        {
            if (_done)
            {
                key = string.Empty;
                json = Array.Empty<byte>();
                return false;
            }

            if (_enumerator.MoveNext())
            {
                var nut = _enumerator.Current;
                key = nut.Id;
                var jsonString = System.Text.Json.JsonSerializer.Serialize(nut.Payload, JsonContext.Default.Object);
                json = System.Text.Encoding.UTF8.GetBytes(jsonString);
                return true;
            }
            else
            {
                _done = true;
                key = string.Empty;
                json = Array.Empty<byte>();
                return false;
            }
        }

        public void Dispose()
        {
            _enumerator.Dispose();
        }
    }

    /// <summary>
    /// Subscription to tree changes. Invokes callback when items are added or modified.
    /// The callback is invoked from a background thread.
    /// </summary>
    internal sealed class JsonSubscription
    {
        private readonly IDisposable _subscription;

        public JsonSubscription(Tree<object> tree, Action<string, byte[]> callback)
        {
            // Subscribe to stash events using Reactive Extensions
            _subscription = tree.ObserveStash().Subscribe(change =>
            {
                try
                {
                    // Serialize the payload to JSON
                    var jsonString = System.Text.Json.JsonSerializer.Serialize(change.Item, JsonContext.Default.Object);
                    var jsonBytes = System.Text.Encoding.UTF8.GetBytes(jsonString);

                    // Invoke the callback on a background thread
                    System.Threading.ThreadPool.QueueUserWorkItem(_ =>
                    {
                        try
                        {
                            callback(change.Id, jsonBytes);
                        }
                        catch
                        {
                            // Ignore exceptions in user callback to prevent crashing the event handler
                        }
                    });
                }
                catch
                {
                    // Ignore serialization errors
                }
            });
        }

        public void Dispose()
        {
            // Unsubscribe from tree events
            _subscription?.Dispose();
        }
    }

    public static JsonTree OpenJsonTree(string uri)
    {
        try
        {
            // Parse URI to determine storage type
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                var tree = new Tree<object>(new FileTrunk<object>(path));
                return new JsonTree(tree);
            }
            else if (uri.StartsWith("memory://"))
            {
                var tree = new Tree<object>(new MemoryTrunk<object>());
                return new JsonTree(tree);
            }
            else
            {
                // Default to file storage with the URI as the path
                var tree = new Tree<object>(new FileTrunk<object>(uri));
                return new JsonTree(tree);
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open tree with URI '{uri}': {ex.Message}", ex);
        }
    }

    /// <summary>
    /// JSON-compatible transaction wrapper around AcornDB TreeTransaction
    /// </summary>
    internal sealed class JsonTransaction
    {
        private readonly TreeTransaction<object> _transaction;

        public JsonTransaction(TreeTransaction<object> transaction)
        {
            _transaction = transaction;
        }

        public void Stash(string id, ReadOnlySpan<byte> json)
        {
            try
            {
                // Parse JSON bytes into object using System.Text.Json with source generation
                var jsonString = System.Text.Encoding.UTF8.GetString(json);
                var obj = System.Text.Json.JsonSerializer.Deserialize(jsonString, JsonContext.Default.Object);
                if (obj != null)
                {
                    _transaction.Stash(id, obj);
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to stash JSON for id '{id}' in transaction: {ex.Message}", ex);
            }
        }

        public void Delete(string id)
        {
            try
            {
                _transaction.Toss(id);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to delete id '{id}' in transaction: {ex.Message}", ex);
            }
        }

        public bool Commit()
        {
            try
            {
                return _transaction.Commit();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to commit transaction: {ex.Message}", ex);
            }
        }

        public void Rollback()
        {
            try
            {
                _transaction.Rollback();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to rollback transaction: {ex.Message}", ex);
            }
        }
    }
}
