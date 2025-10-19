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
using AcornDB.Sync;
using AcornDB.Security;
using AcornDB.Compression;
using System.IO.Compression;

internal static class AcornFacade
{
    // Real AcornDB integration using Tree<object> for JSON storage
    // This avoids JsonElement serialization issues with Newtonsoft.Json
    internal sealed class JsonTree
    {
        internal readonly Tree<object> _tree;

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

    /// <summary>
    /// JSON-compatible mesh coordinator for advanced sync operations
    /// </summary>
    internal sealed class JsonMesh
    {
        private readonly MeshCoordinator<object> _meshCoordinator;

        public JsonMesh()
        {
            _meshCoordinator = new MeshCoordinator<object>();
        }

        public void AddNode(string nodeId, JsonTree tree)
        {
            try
            {
                _meshCoordinator.AddNode(nodeId, tree._tree);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to add node '{nodeId}' to mesh: {ex.Message}", ex);
            }
        }

        public void ConnectNodes(string nodeA, string nodeB)
        {
            try
            {
                _meshCoordinator.ConnectNodes(nodeA, nodeB);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to connect nodes '{nodeA}' and '{nodeB}': {ex.Message}", ex);
            }
        }

        public void CreateFullMesh()
        {
            try
            {
                _meshCoordinator.CreateFullMesh();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to create full mesh: {ex.Message}", ex);
            }
        }

        public void CreateRing()
        {
            try
            {
                _meshCoordinator.CreateRing();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to create ring topology: {ex.Message}", ex);
            }
        }

        public void CreateStar(string hubNodeId)
        {
            try
            {
                _meshCoordinator.CreateStar(hubNodeId);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to create star topology with hub '{hubNodeId}': {ex.Message}", ex);
            }
        }

        public void SynchronizeAll()
        {
            try
            {
                _meshCoordinator.SynchronizeAll();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to synchronize mesh: {ex.Message}", ex);
            }
        }
    }

    /// <summary>
    /// JSON-compatible peer-to-peer sync connection
    /// </summary>
    internal sealed class JsonP2P
    {
        private readonly Tree<object> _localTree;
        private readonly Tree<object> _remoteTree;
        private SyncMode _syncMode = SyncMode.Bidirectional;
        private ConflictDirection _conflictDirection = ConflictDirection.UseJudge;

        public JsonP2P(JsonTree localTree, JsonTree remoteTree)
        {
            _localTree = localTree._tree;
            _remoteTree = remoteTree._tree;
        }

        public void SyncBidirectional()
        {
            try
            {
                if (_syncMode == SyncMode.Disabled) return;

                // Create bidirectional sync using InProcessBranch
                var localBranch = new InProcessBranch<object>(_localTree);
                var remoteBranch = new InProcessBranch<object>(_remoteTree);

                // Sync local -> remote
                if (_syncMode == SyncMode.Bidirectional || _syncMode == SyncMode.PushOnly)
                {
                    foreach (var nut in _localTree.ExportChanges())
                    {
                        remoteBranch.TryPush(nut.Id, nut);
                    }
                }

                // Sync remote -> local
                if (_syncMode == SyncMode.Bidirectional || _syncMode == SyncMode.PullOnly)
                {
                    foreach (var nut in _remoteTree.ExportChanges())
                    {
                        localBranch.TryPush(nut.Id, nut);
                    }
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to sync bidirectionally: {ex.Message}", ex);
            }
        }

        public void SyncPushOnly()
        {
            try
            {
                if (_syncMode == SyncMode.Disabled || _syncMode == SyncMode.PullOnly) return;

                var remoteBranch = new InProcessBranch<object>(_localTree);
                foreach (var nut in _localTree.ExportChanges())
                {
                    remoteBranch.TryPush(nut.Id, nut);
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to sync push-only: {ex.Message}", ex);
            }
        }

        public void SyncPullOnly()
        {
            try
            {
                if (_syncMode == SyncMode.Disabled || _syncMode == SyncMode.PushOnly) return;

                var localBranch = new InProcessBranch<object>(_remoteTree);
                foreach (var nut in _remoteTree.ExportChanges())
                {
                    localBranch.TryPush(nut.Id, nut);
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to sync pull-only: {ex.Message}", ex);
            }
        }

        public void SetSyncMode(int syncMode)
        {
            _syncMode = syncMode switch
            {
                0 => SyncMode.Bidirectional,
                1 => SyncMode.PushOnly,
                2 => SyncMode.PullOnly,
                3 => SyncMode.Disabled,
                _ => throw new ArgumentException($"Invalid sync mode: {syncMode}")
            };
        }

        public void SetConflictDirection(int conflictDirection)
        {
            _conflictDirection = conflictDirection switch
            {
                0 => ConflictDirection.UseJudge,
                1 => ConflictDirection.PreferLocal,
                2 => ConflictDirection.PreferRemote,
                _ => throw new ArgumentException($"Invalid conflict direction: {conflictDirection}")
            };
        }
    }

    // Factory methods for Advanced Sync
    public static JsonMesh CreateMesh()
    {
        return new JsonMesh();
    }

    public static JsonP2P CreateP2P(JsonTree localTree, JsonTree remoteTree)
    {
        return new JsonP2P(localTree, remoteTree);
    }

    /// <summary>
    /// Encryption provider wrapper for FFI
    /// </summary>
    internal sealed class JsonEncryptionProvider
    {
        private readonly IEncryptionProvider _encryption;

        public JsonEncryptionProvider(IEncryptionProvider encryption)
        {
            _encryption = encryption ?? throw new ArgumentNullException(nameof(encryption));
        }

        public string Encrypt(string plaintext)
        {
            try
            {
                return _encryption.Encrypt(plaintext);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to encrypt data: {ex.Message}", ex);
            }
        }

        public string Decrypt(string ciphertext)
        {
            try
            {
                return _encryption.Decrypt(ciphertext);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to decrypt data: {ex.Message}", ex);
            }
        }

        public bool IsEnabled => _encryption.IsEnabled;

        public string ExportKeyBase64()
        {
            if (_encryption is AesEncryptionProvider aes)
            {
                return aes.ExportKeyBase64();
            }
            throw new NotSupportedException("Key export not supported for this encryption provider");
        }

        public string ExportIVBase64()
        {
            if (_encryption is AesEncryptionProvider aes)
            {
                return aes.ExportIVBase64();
            }
            throw new NotSupportedException("IV export not supported for this encryption provider");
        }
    }

    internal sealed class JsonCompressionProvider
    {
        private readonly ICompressionProvider _compression;

        public JsonCompressionProvider(ICompressionProvider compression)
        {
            _compression = compression ?? throw new ArgumentNullException(nameof(compression));
        }

        public string Compress(string data)
        {
            try
            {
                var bytes = Encoding.UTF8.GetBytes(data);
                var compressed = _compression.Compress(bytes);
                return Convert.ToBase64String(compressed);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Compression failed: {ex.Message}", ex);
            }
        }

        public string Decompress(string compressedData)
        {
            try
            {
                var compressedBytes = Convert.FromBase64String(compressedData);
                var decompressed = _compression.Decompress(compressedBytes);
                return Encoding.UTF8.GetString(decompressed);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Decompression failed: {ex.Message}", ex);
            }
        }

        public bool IsEnabled => _compression.IsEnabled;

        public string AlgorithmName => _compression.AlgorithmName;

        public CompressionStats GetStats(string originalData, string compressedData)
        {
            try
            {
                var originalBytes = Encoding.UTF8.GetBytes(originalData);
                var compressedBytes = Convert.FromBase64String(compressedData);
                
                return new CompressionStats
                {
                    OriginalSize = originalBytes.Length,
                    CompressedSize = compressedBytes.Length,
                    Ratio = originalBytes.Length > 0 ? (double)compressedBytes.Length / originalBytes.Length : 1.0,
                    SpaceSaved = originalBytes.Length - compressedBytes.Length
                };
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Stats calculation failed: {ex.Message}", ex);
            }
        }
    }

    internal sealed class CompressionStats
    {
        public int OriginalSize { get; set; }
        public int CompressedSize { get; set; }
        public double Ratio { get; set; }
        public int SpaceSaved { get; set; }
    }

    internal sealed class JsonCacheStrategy
    {
        private readonly ICacheStrategy<object> _cacheStrategy;

        public JsonCacheStrategy(ICacheStrategy<object> cacheStrategy)
        {
            _cacheStrategy = cacheStrategy ?? throw new ArgumentNullException(nameof(cacheStrategy));
        }

        public void Reset()
        {
            try
            {
                _cacheStrategy.Reset();
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Cache reset failed: {ex.Message}", ex);
            }
        }

        public CacheStats GetStats()
        {
            try
            {
                if (_cacheStrategy is LRUCacheStrategy<object> lru)
                {
                    var stats = lru.GetStats();
                    return new CacheStats
                    {
                        TrackedItems = stats.TrackedItems,
                        MaxSize = stats.MaxSize,
                        UtilizationPercentage = stats.UtilizationPercentage
                    };
                }
                else if (_cacheStrategy is NoEvictionStrategy<object>)
                {
                    return new CacheStats
                    {
                        TrackedItems = 0, // NoEviction doesn't track items
                        MaxSize = int.MaxValue, // Unlimited
                        UtilizationPercentage = 0.0
                    };
                }
                else
                {
                    return new CacheStats
                    {
                        TrackedItems = 0,
                        MaxSize = 0,
                        UtilizationPercentage = 0.0
                    };
                }
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to get cache stats: {ex.Message}", ex);
            }
        }

        public bool IsEvictionEnabled
        {
            get
            {
                // LRU cache always has eviction enabled, NoEviction never does
                return _cacheStrategy is LRUCacheStrategy<object>;
            }
        }

        public string StrategyName
        {
            get
            {
                return _cacheStrategy switch
                {
                    LRUCacheStrategy<object> => "LRU",
                    NoEvictionStrategy<object> => "NoEviction",
                    _ => "Unknown"
                };
            }
        }
    }

    internal sealed class CacheStats
    {
        public int TrackedItems { get; set; }
        public int MaxSize { get; set; }
        public double UtilizationPercentage { get; set; }
    }

    internal sealed class JsonConflictJudge
    {
        private readonly IConflictJudge<object> _conflictJudge;

        public JsonConflictJudge(IConflictJudge<object> conflictJudge)
        {
            _conflictJudge = conflictJudge ?? throw new ArgumentNullException(nameof(conflictJudge));
        }

        public string ResolveConflict(string localJson, string incomingJson)
        {
            try
            {
                var localNut = JsonConvert.DeserializeObject<Nut<object>>(localJson);
                var incomingNut = JsonConvert.DeserializeObject<Nut<object>>(incomingJson);
                
                if (localNut == null || incomingNut == null)
                {
                    throw new InvalidOperationException("Failed to deserialize nuts");
                }
                
                var winner = _conflictJudge.Judge(localNut, incomingNut);
                return JsonConvert.SerializeObject(winner);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Conflict resolution failed: {ex.Message}", ex);
            }
        }

        public string JudgeName
        {
            get
            {
                return _conflictJudge switch
                {
                    TimestampJudge<object> => "Timestamp",
                    VersionJudge<object> => "Version",
                    LocalWinsJudge<object> => "LocalWins",
                    RemoteWinsJudge<object> => "RemoteWins",
                    _ => "Unknown"
                };
            }
        }
    }

    // Factory methods for Encryption
    public static JsonEncryptionProvider CreateEncryptionFromPassword(string password, string salt)
    {
        try
        {
            var encryption = AesEncryptionProvider.FromPassword(password, salt);
            return new JsonEncryptionProvider(encryption);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create encryption from password: {ex.Message}", ex);
        }
    }

    public static JsonEncryptionProvider CreateEncryptionFromKeyIV(string keyBase64, string ivBase64)
    {
        try
        {
            var key = Convert.FromBase64String(keyBase64);
            var iv = Convert.FromBase64String(ivBase64);
            var encryption = new AesEncryptionProvider(key, iv);
            return new JsonEncryptionProvider(encryption);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create encryption from key/IV: {ex.Message}", ex);
        }
    }

    public static (string keyBase64, string ivBase64) GenerateKeyAndIV()
    {
        try
        {
            var (key, iv) = AesEncryptionProvider.GenerateKeyAndIV();
            return (Convert.ToBase64String(key), Convert.ToBase64String(iv));
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to generate key and IV: {ex.Message}", ex);
        }
    }

    // Factory methods for Compression
    public static JsonCompressionProvider CreateGzipCompression(CompressionLevel compressionLevel)
    {
        try
        {
            var compression = new GzipCompressionProvider(compressionLevel);
            return new JsonCompressionProvider(compression);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create Gzip compression: {ex.Message}", ex);
        }
    }

    public static JsonCompressionProvider CreateBrotliCompression(CompressionLevel compressionLevel)
    {
        try
        {
            var compression = new BrotliCompressionProvider(compressionLevel);
            return new JsonCompressionProvider(compression);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create Brotli compression: {ex.Message}", ex);
        }
    }

    public static JsonCompressionProvider CreateNoCompression()
    {
        try
        {
            var compression = new NoCompressionProvider();
            return new JsonCompressionProvider(compression);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create no compression: {ex.Message}", ex);
        }
    }

    // Factory methods for Cache
    public static JsonCacheStrategy CreateLRUCache(int maxSize)
    {
        try
        {
            var cacheStrategy = new LRUCacheStrategy<object>(maxSize);
            return new JsonCacheStrategy(cacheStrategy);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create LRU cache: {ex.Message}", ex);
        }
    }

    public static JsonCacheStrategy CreateNoEvictionCache()
    {
        try
        {
            var cacheStrategy = new NoEvictionStrategy<object>();
            return new JsonCacheStrategy(cacheStrategy);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create no eviction cache: {ex.Message}", ex);
        }
    }

    public static JsonTree OpenJsonTreeWithCache(string uri, JsonCacheStrategy cacheStrategy)
    {
        try
        {
            // Parse URI to determine storage type
            ITrunk<object> baseTrunk;
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                baseTrunk = new FileTrunk<object>(path);
            }
            else if (uri.StartsWith("memory://"))
            {
                baseTrunk = new MemoryTrunk<object>();
            }
            else
            {
                // Default to file storage with the URI as the path
                baseTrunk = new FileTrunk<object>(uri);
            }

            // Create tree with custom cache strategy
            var tree = new Tree<object>(baseTrunk, cacheStrategy._cacheStrategy);
            return new JsonTree(tree);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open tree with cache strategy '{uri}': {ex.Message}", ex);
        }
    }

    // Factory methods for Conflict Resolution
    public static JsonConflictJudge CreateTimestampJudge()
    {
        try
        {
            var judge = new TimestampJudge<object>();
            return new JsonConflictJudge(judge);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create timestamp judge: {ex.Message}", ex);
        }
    }

    public static JsonConflictJudge CreateVersionJudge()
    {
        try
        {
            var judge = new VersionJudge<object>();
            return new JsonConflictJudge(judge);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create version judge: {ex.Message}", ex);
        }
    }

    public static JsonConflictJudge CreateLocalWinsJudge()
    {
        try
        {
            var judge = new LocalWinsJudge<object>();
            return new JsonConflictJudge(judge);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create local wins judge: {ex.Message}", ex);
        }
    }

    public static JsonConflictJudge CreateRemoteWinsJudge()
    {
        try
        {
            var judge = new RemoteWinsJudge<object>();
            return new JsonConflictJudge(judge);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to create remote wins judge: {ex.Message}", ex);
        }
    }

    public static JsonTree OpenJsonTreeWithConflictJudge(string uri, JsonConflictJudge conflictJudge)
    {
        try
        {
            // Parse URI to determine storage type
            ITrunk<object> baseTrunk;
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                baseTrunk = new FileTrunk<object>(path);
            }
            else if (uri.StartsWith("memory://"))
            {
                baseTrunk = new MemoryTrunk<object>();
            }
            else
            {
                // Default to file storage with the URI as the path
                baseTrunk = new FileTrunk<object>(uri);
            }

            // Create tree with custom conflict judge
            var tree = new Tree<object>(baseTrunk, null, conflictJudge._conflictJudge);
            return new JsonTree(tree);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open tree with conflict judge '{uri}': {ex.Message}", ex);
        }
    }

    public static JsonTree OpenJsonTreeCompressed(string uri, JsonCompressionProvider compression)
    {
        try
        {
            // Parse URI to determine storage type
            ITrunk<object> baseTrunk;
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                baseTrunk = new FileTrunk<object>(path);
            }
            else if (uri.StartsWith("memory://"))
            {
                baseTrunk = new MemoryTrunk<object>();
            }
            else
            {
                // Default to file storage with the URI as the path
                baseTrunk = new FileTrunk<object>(uri);
            }

            // Wrap with compression
            var compressedTrunk = new CompressedTrunk<object>(baseTrunk, compression._compression);
            var tree = new Tree<object>(compressedTrunk);
            return new JsonTree(tree);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open compressed tree with URI '{uri}': {ex.Message}", ex);
        }
    }

    public static JsonTree OpenJsonTreeEncrypted(string uri, JsonEncryptionProvider encryption)
    {
        try
        {
            // Parse URI to determine storage type
            ITrunk<object> baseTrunk;
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                baseTrunk = new FileTrunk<object>(path);
            }
            else if (uri.StartsWith("memory://"))
            {
                baseTrunk = new MemoryTrunk<object>();
            }
            else
            {
                // Default to file storage with the URI as the path
                baseTrunk = new FileTrunk<object>(uri);
            }

            // Wrap with encryption
            var encryptedTrunk = new EncryptedTrunk<object>(baseTrunk, encryption._encryption);
            var tree = new Tree<object>(encryptedTrunk);
            return new JsonTree(tree);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open encrypted tree with URI '{uri}': {ex.Message}", ex);
        }
    }

    public static JsonTree OpenJsonTreeEncryptedCompressed(string uri, JsonEncryptionProvider encryption, CompressionLevel compressionLevel)
    {
        try
        {
            // Parse URI to determine storage type
            ITrunk<object> baseTrunk;
            if (uri.StartsWith("file://"))
            {
                var path = uri.Substring(7); // Remove "file://" prefix
                baseTrunk = new FileTrunk<object>(path);
            }
            else if (uri.StartsWith("memory://"))
            {
                baseTrunk = new MemoryTrunk<object>();
            }
            else
            {
                // Default to file storage with the URI as the path
                baseTrunk = new FileTrunk<object>(uri);
            }

            // Wrap with compression first, then encryption
            var compressionProvider = new GzipCompressionProvider(compressionLevel);
            var compressedTrunk = new CompressedTrunk<object>(baseTrunk, compressionProvider);
            var encryptedTrunk = new EncryptedTrunk<object>(compressedTrunk, encryption._encryption);
            var tree = new Tree<object>(encryptedTrunk);
            return new JsonTree(tree);
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Failed to open encrypted compressed tree with URI '{uri}': {ex.Message}", ex);
        }
    }
}
