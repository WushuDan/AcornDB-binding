using System;
// using AcornDB; // TODO: add reference to actual Acorn types

internal static class AcornFacade
{
    // NOTE: Replace this stub with real AcornDB integration.
    internal sealed class JsonTree
    {
        public void Stash(string id, ReadOnlySpan<byte> json)
        {
            // TODO: call into real Acorn Tree API to upsert a JSON blob by id.
            // This stub does nothing.
        }

        public byte[]? Crack(string id)
        {
            // TODO: retrieve JSON blob by id
            return null;
        }
    }

    public static JsonTree OpenJsonTree(string uri)
    {
        // TODO: use Acorn nursery/trunk resolver based on URI (file:, git:, s3:, pg:)
        return new JsonTree();
    }
}
