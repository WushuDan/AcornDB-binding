using System;
using System.Text.Json;
using AcornDB;
using AcornDB.Storage;

internal static class AcornFacade
{
    // Real AcornDB integration using Tree<JsonElement> for JSON storage
    // JsonElement is NativeAOT-compatible and can represent any JSON structure
    internal sealed class JsonTree
    {
        private readonly Tree<JsonElement> _tree;

        public JsonTree(Tree<JsonElement> tree)
        {
            _tree = tree;
        }

        public void Stash(string id, ReadOnlySpan<byte> json)
        {
            try
            {
                // Parse JSON bytes into JsonElement using source-generated context
                var jsonElement = JsonSerializer.Deserialize(json, JsonContext.Default.JsonElement);
                _tree.Stash(id, jsonElement);
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
                var element = _tree.Crack(id);
                if (element.ValueKind == JsonValueKind.Undefined) return null;

                // Serialize JsonElement back to bytes using source-generated context
                return JsonSerializer.SerializeToUtf8Bytes(element, JsonContext.Default.JsonElement);
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
                var element = _tree.Crack(id);
                return element.ValueKind != JsonValueKind.Undefined;
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
    }

    public static JsonTree OpenJsonTree(string uri)
    {
        try
        {
            // Parse URI to determine storage type
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                var tree = new Tree<JsonElement>(new FileTrunk<JsonElement>(path));
                return new JsonTree(tree);
            }
            else if (uri.StartsWith("memory://"))
            {
                var tree = new Tree<JsonElement>(new MemoryTrunk<JsonElement>());
                return new JsonTree(tree);
            }
            else
            {
                // Default to file storage with the URI as the path
                var tree = new Tree<JsonElement>(new FileTrunk<JsonElement>(uri));
                return new JsonTree(tree);
            }
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open tree with URI '{uri}': {ex.Message}", ex);
        }
    }
}
