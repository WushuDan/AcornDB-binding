using System;
using AcornDB;
using AcornDB.Storage;

internal static class AcornFacade
{
    // Real AcornDB integration using Tree<object> for JSON storage
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
                var jsonString = System.Text.Encoding.UTF8.GetString(json);
                var obj = System.Text.Json.JsonSerializer.Deserialize<object>(jsonString);
                _tree.Stash(id, obj);
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
                
                var jsonString = System.Text.Json.JsonSerializer.Serialize(obj);
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
                return _tree.Crack(id) != null;
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
                return _tree.Nuts.Count();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to count items: {ex.Message}", ex);
            }
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
                var tree = new Tree<object>(path);
                return new JsonTree(tree);
            }
            else if (uri.StartsWith("memory://"))
            {
                var tree = new Tree<object>(); // Default to in-memory
                return new JsonTree(tree);
            }
            else
            {
                // Default to file storage with the URI as the path
                var tree = new Tree<object>(uri);
                return new JsonTree(tree);
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open tree with URI '{uri}': {ex.Message}", ex);
        }
    }
}
