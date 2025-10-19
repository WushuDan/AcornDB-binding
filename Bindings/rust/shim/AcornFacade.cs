using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using AcornDB;
using AcornDB.Storage;
using AcornDB.Shim;

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
}
