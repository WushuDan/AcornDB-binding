using System;
using System.Runtime.InteropServices;

public static class NativeExports
{
    static readonly HandleTable<AcornFacade.JsonTree> Trees = new();
    static readonly HandleTable<AcornFacade.JsonIterator> Iterators = new();
    static readonly HandleTable<AcornFacade.JsonSubscription> Subscriptions = new();
    static readonly HandleTable<SubscriptionContext> SubscriptionContexts = new();
    static readonly HandleTable<AcornFacade.JsonTransaction> Transactions = new();
    static readonly HandleTable<AcornFacade.JsonMesh> Meshes = new();
    static readonly HandleTable<AcornFacade.JsonP2P> P2PConnections = new();
    static readonly HandleTable<AcornFacade.JsonEncryptionProvider> EncryptionProviders = new();
    static readonly HandleTable<AcornFacade.JsonCompressionProvider> CompressionProviders = new();

    [UnmanagedCallersOnly(EntryPoint = "acorn_open_tree")]
    public static int OpenTree(IntPtr uriUtf8, IntPtr handlePtr)
    {
        try {
            string uri = Utf8.In(uriUtf8);
            var tree = AcornFacade.OpenJsonTree(uri);
            ulong handle = Trees.Add(tree);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_close_tree")]
    public static int CloseTree(ulong handle)
    {
        try {
            Trees.Remove(handle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_stash_json")]
    public static unsafe int Stash(ulong handle, IntPtr idUtf8, IntPtr data, nuint len)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            var span = new ReadOnlySpan<byte>(data.ToPointer(), checked((int)len));
            tree.Stash(id, span);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct AcornBuf { public unsafe byte* data; public nuint len; }

    [UnmanagedCallersOnly(EntryPoint = "acorn_crack_json")]
    public static unsafe int Crack(ulong handle, IntPtr idUtf8, IntPtr outBufPtr)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            var bytes = tree.Crack(id);
            if (bytes is null) return 1; // not found
            
            // Allocate unmanaged memory and copy the data
            var mem = (byte*)NativeMemory.Alloc((nuint)bytes.Length);
            if (mem == null) {
                Error.Set("Failed to allocate memory for JSON data");
                return -1;
            }
            
            try {
                fixed (byte* p = bytes) {
                    new ReadOnlySpan<byte>(p, bytes.Length).CopyTo(new Span<byte>(mem, bytes.Length));
                }
                var buf = new AcornBuf { data = mem, len = (nuint)bytes.Length };
                *(AcornBuf*)outBufPtr = buf;
                return 0;
            } catch {
                // If copying fails, free the allocated memory
                NativeMemory.Free(mem);
                throw;
            }
        } catch (Exception ex) { 
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_delete")]
    public static int Delete(ulong handle, IntPtr idUtf8)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            tree.Delete(id);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_exists")]
    public static int Exists(ulong handle, IntPtr idUtf8)
    {
        try {
            var tree = Trees.Get(handle);
            string id = Utf8.In(idUtf8);
            return tree.Exists(id) ? 1 : 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_count")]
    public static int Count(ulong handle, IntPtr countPtr)
    {
        try {
            var tree = Trees.Get(handle);
            nuint count = (nuint)tree.Count();
            unsafe { *(nuint*)countPtr = count; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(nuint*)countPtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_free_buf")]
    public static unsafe void FreeBuf(IntPtr bufPtr)
    {
        var buf = *(AcornBuf*)bufPtr;
        if (buf.data != null) {
            NativeMemory.Free(buf.data);
            buf.data = null;
            buf.len = 0;
            *(AcornBuf*)bufPtr = buf;
        }
    }

    // Iterator exports

    [UnmanagedCallersOnly(EntryPoint = "acorn_iter_start")]
    public static int IterStart(ulong treeHandle, IntPtr prefixUtf8, IntPtr outIterPtr)
    {
        try {
            var tree = Trees.Get(treeHandle);
            string prefix = Utf8.In(prefixUtf8);
            var iterator = tree.CreateIterator(prefix);
            ulong iterHandle = Iterators.Add(iterator);
            unsafe { *(ulong*)outIterPtr = iterHandle; }
            return 0;
        } catch (Exception ex) {
            unsafe { *(ulong*)outIterPtr = 0; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_iter_next")]
    public static unsafe int IterNext(ulong iterHandle, IntPtr outKeyPtr, IntPtr outJsonPtr, IntPtr outDonePtr)
    {
        try {
            var iterator = Iterators.Get(iterHandle);

            bool hasItem = iterator.Next(out string key, out byte[] json);

            // Set done flag
            *(int*)outDonePtr = hasItem ? 0 : 1;

            if (hasItem)
            {
                // Allocate and copy key
                var keyBytes = System.Text.Encoding.UTF8.GetBytes(key);
                var keyMem = (byte*)NativeMemory.Alloc((nuint)keyBytes.Length);
                if (keyMem == null) {
                    Error.Set("Failed to allocate memory for key");
                    return -1;
                }
                fixed (byte* p = keyBytes) {
                    new ReadOnlySpan<byte>(p, keyBytes.Length).CopyTo(new Span<byte>(keyMem, keyBytes.Length));
                }
                var keyBuf = new AcornBuf { data = keyMem, len = (nuint)keyBytes.Length };
                *(AcornBuf*)outKeyPtr = keyBuf;

                // Allocate and copy JSON
                var jsonMem = (byte*)NativeMemory.Alloc((nuint)json.Length);
                if (jsonMem == null) {
                    NativeMemory.Free(keyMem); // Clean up key memory
                    Error.Set("Failed to allocate memory for JSON");
                    return -1;
                }
                fixed (byte* p = json) {
                    new ReadOnlySpan<byte>(p, json.Length).CopyTo(new Span<byte>(jsonMem, json.Length));
                }
                var jsonBuf = new AcornBuf { data = jsonMem, len = (nuint)json.Length };
                *(AcornBuf*)outJsonPtr = jsonBuf;
            }
            else
            {
                // No more items - set empty buffers
                *(AcornBuf*)outKeyPtr = new AcornBuf { data = null, len = 0 };
                *(AcornBuf*)outJsonPtr = new AcornBuf { data = null, len = 0 };
            }

            return 0;
        } catch (Exception ex) {
            unsafe { *(int*)outDonePtr = 1; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_iter_close")]
    public static int IterClose(ulong iterHandle)
    {
        try {
            if (Iterators.Remove(iterHandle, out var iterator))
            {
                iterator?.Dispose();
            }
            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }

    // Subscription exports

    // Callback delegate matching the C signature
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void AcornEventCallback(IntPtr key, IntPtr json, nuint len, IntPtr user);

    // Context to hold callback and user data
    private class SubscriptionContext
    {
        public AcornEventCallback Callback { get; set; }
        public IntPtr UserData { get; set; }
        public AcornFacade.JsonSubscription? Subscription { get; set; }

        public SubscriptionContext(AcornEventCallback callback, IntPtr userData)
        {
            Callback = callback;
            UserData = userData;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_subscribe")]
    public static int Subscribe(ulong treeHandle, IntPtr callbackPtr, IntPtr userData, IntPtr outSubPtr)
    {
        try {
            var tree = Trees.Get(treeHandle);

            // Marshal the function pointer to a delegate
            var callback = Marshal.GetDelegateForFunctionPointer<AcornEventCallback>(callbackPtr);

            // Create subscription context
            var context = new SubscriptionContext(callback, userData);

            // Create the subscription with a wrapper that calls the native callback
            var subscription = tree.Subscribe((key, jsonBytes) =>
            {
                unsafe
                {
                    // Allocate unmanaged memory for key
                    var keyBytes = System.Text.Encoding.UTF8.GetBytes(key);
                    var keyMem = (byte*)NativeMemory.Alloc((nuint)keyBytes.Length + 1); // +1 for null terminator
                    fixed (byte* p = keyBytes)
                    {
                        new ReadOnlySpan<byte>(p, keyBytes.Length).CopyTo(new Span<byte>(keyMem, keyBytes.Length));
                    }
                    keyMem[keyBytes.Length] = 0; // Null terminator

                    // Allocate unmanaged memory for JSON
                    var jsonMem = (byte*)NativeMemory.Alloc((nuint)jsonBytes.Length);
                    fixed (byte* p = jsonBytes)
                    {
                        new ReadOnlySpan<byte>(p, jsonBytes.Length).CopyTo(new Span<byte>(jsonMem, jsonBytes.Length));
                    }

                    try
                    {
                        // Invoke the callback
                        context.Callback((IntPtr)keyMem, (IntPtr)jsonMem, (nuint)jsonBytes.Length, context.UserData);
                    }
                    finally
                    {
                        // Free the allocated memory
                        // Note: The callback must copy the data if it needs it after returning
                        NativeMemory.Free(keyMem);
                        NativeMemory.Free(jsonMem);
                    }
                }
            });

            context.Subscription = subscription;

            // Store context and return handle
            ulong subHandle = SubscriptionContexts.Add(context);
            unsafe { *(ulong*)outSubPtr = subHandle; }

            return 0;
        } catch (Exception ex) {
            unsafe { *(ulong*)outSubPtr = 0; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_unsubscribe")]
    public static int Unsubscribe(ulong subHandle)
    {
        try {
            if (SubscriptionContexts.Remove(subHandle, out var context))
            {
                context?.Subscription?.Dispose();
            }
            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }

    // Sync export

    [UnmanagedCallersOnly(EntryPoint = "acorn_sync_http")]
    public static int SyncHttp(ulong treeHandle, IntPtr urlUtf8)
    {
        try {
            var tree = Trees.Get(treeHandle);
            string url = Utf8.In(urlUtf8);

            // SyncHttpAsync is async, but we need to block for FFI
            // Use GetAwaiter().GetResult() to synchronously wait
            tree.SyncHttpAsync(url).GetAwaiter().GetResult();

            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }

    // Batch operations exports

    [UnmanagedCallersOnly(EntryPoint = "acorn_batch_stash")]
    public static unsafe int BatchStash(ulong treeHandle, IntPtr* idsPtr, IntPtr* jsonsPtr, nuint* jsonLensPtr, nuint count)
    {
        try {
            var tree = Trees.Get(treeHandle);
            int itemCount = checked((int)count);

            // Convert arrays from C to C#
            var ids = new string[itemCount];
            var jsons = new byte[itemCount][];

            for (int i = 0; i < itemCount; i++)
            {
                ids[i] = Utf8.In(idsPtr[i]);

                int jsonLen = checked((int)jsonLensPtr[i]);
                jsons[i] = new byte[jsonLen];

                var jsonSpan = new ReadOnlySpan<byte>((void*)jsonsPtr[i], jsonLen);
                jsonSpan.CopyTo(jsons[i]);
            }

            tree.BatchStash(ids, jsons);
            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_batch_crack")]
    public static unsafe int BatchCrack(ulong treeHandle, IntPtr* idsPtr, nuint count, IntPtr outJsonsPtr, IntPtr outFoundPtr)
    {
        try {
            var tree = Trees.Get(treeHandle);
            int itemCount = checked((int)count);

            // Convert IDs from C to C#
            var ids = new string[itemCount];
            for (int i = 0; i < itemCount; i++)
            {
                ids[i] = Utf8.In(idsPtr[i]);
            }

            // Call batch crack
            var results = tree.BatchCrack(ids);

            // Convert results to C buffers
            var outJsons = (AcornBuf*)outJsonsPtr;
            var outFound = (int*)outFoundPtr;

            for (int i = 0; i < itemCount; i++)
            {
                if (results[i] == null)
                {
                    outJsons[i] = new AcornBuf { data = null, len = 0 };
                    outFound[i] = 0;
                }
                else
                {
                    var jsonBytes = results[i];
                    var mem = (byte*)NativeMemory.Alloc((nuint)jsonBytes.Length);
                    if (mem == null)
                    {
                        // Clean up previously allocated buffers
                        for (int j = 0; j < i; j++)
                        {
                            if (outJsons[j].data != null)
                            {
                                NativeMemory.Free(outJsons[j].data);
                            }
                        }
                        Error.Set("Failed to allocate memory for JSON data");
                        return -1;
                    }

                    fixed (byte* p = jsonBytes)
                    {
                        new ReadOnlySpan<byte>(p, jsonBytes.Length).CopyTo(new Span<byte>(mem, jsonBytes.Length));
                    }

                    outJsons[i] = new AcornBuf { data = mem, len = (nuint)jsonBytes.Length };
                    outFound[i] = 1;
                }
            }

            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_batch_delete")]
    public static unsafe int BatchDelete(ulong treeHandle, IntPtr* idsPtr, nuint count)
    {
        try {
            var tree = Trees.Get(treeHandle);
            int itemCount = checked((int)count);

            // Convert IDs from C to C#
            var ids = new string[itemCount];
            for (int i = 0; i < itemCount; i++)
            {
                ids[i] = Utf8.In(idsPtr[i]);
            }

            tree.BatchDelete(ids);
            return 0;
        } catch (Exception ex) {
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_begin_transaction")]
    public static int BeginTransaction(ulong treeHandle, IntPtr outTransactionPtr)
    {
        try {
            var tree = Trees.Get(treeHandle);
            var transaction = tree.BeginTransaction();
            ulong transactionHandle = Transactions.Add(transaction);
            unsafe { *(ulong*)outTransactionPtr = transactionHandle; }
            return 0;
        } catch (Exception ex) {
            unsafe { *(ulong*)outTransactionPtr = 0; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_transaction_stash")]
    public static unsafe int TransactionStash(ulong transactionHandle, IntPtr idUtf8, IntPtr data, nuint len)
    {
        try {
            var transaction = Transactions.Get(transactionHandle);
            string id = Utf8.In(idUtf8);
            var span = new ReadOnlySpan<byte>(data.ToPointer(), checked((int)len));
            transaction.Stash(id, span);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_transaction_delete")]
    public static int TransactionDelete(ulong transactionHandle, IntPtr idUtf8)
    {
        try {
            var transaction = Transactions.Get(transactionHandle);
            string id = Utf8.In(idUtf8);
            transaction.Delete(id);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_transaction_commit")]
    public static int TransactionCommit(ulong transactionHandle)
    {
        try {
            var transaction = Transactions.Get(transactionHandle);
            bool success = transaction.Commit();
            return success ? 0 : -1;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_transaction_rollback")]
    public static int TransactionRollback(ulong transactionHandle)
    {
        try {
            var transaction = Transactions.Get(transactionHandle);
            transaction.Rollback();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_transaction_close")]
    public static int TransactionClose(ulong transactionHandle)
    {
        try {
            Transactions.Remove(transactionHandle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    // Advanced Sync - Mesh Sync Methods
    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_create")]
    public static int MeshCreate(IntPtr outMeshPtr)
    {
        try {
            var mesh = AcornFacade.CreateMesh();
            ulong meshHandle = Meshes.Add(mesh);
            unsafe { *(ulong*)outMeshPtr = meshHandle; }
            return 0;
        } catch (Exception ex) {
            unsafe { *(ulong*)outMeshPtr = 0; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_add_node")]
    public static int MeshAddNode(ulong meshHandle, IntPtr nodeIdUtf8, ulong treeHandle)
    {
        try {
            var mesh = Meshes.Get(meshHandle);
            var tree = Trees.Get(treeHandle);
            string nodeId = Utf8.In(nodeIdUtf8);
            mesh.AddNode(nodeId, tree);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_connect_nodes")]
    public static int MeshConnectNodes(ulong meshHandle, IntPtr nodeAUtf8, IntPtr nodeBUtf8)
    {
        try {
            var mesh = Meshes.Get(meshHandle);
            string nodeA = Utf8.In(nodeAUtf8);
            string nodeB = Utf8.In(nodeBUtf8);
            mesh.ConnectNodes(nodeA, nodeB);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_create_full_mesh")]
    public static int MeshCreateFullMesh(ulong meshHandle)
    {
        try {
            var mesh = Meshes.Get(meshHandle);
            mesh.CreateFullMesh();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_create_ring")]
    public static int MeshCreateRing(ulong meshHandle)
    {
        try {
            var mesh = Meshes.Get(meshHandle);
            mesh.CreateRing();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_create_star")]
    public static int MeshCreateStar(ulong meshHandle, IntPtr hubNodeIdUtf8)
    {
        try {
            var mesh = Meshes.Get(meshHandle);
            string hubNodeId = Utf8.In(hubNodeIdUtf8);
            mesh.CreateStar(hubNodeId);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_synchronize_all")]
    public static int MeshSynchronizeAll(ulong meshHandle)
    {
        try {
            var mesh = Meshes.Get(meshHandle);
            mesh.SynchronizeAll();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_mesh_close")]
    public static int MeshClose(ulong meshHandle)
    {
        try {
            Meshes.Remove(meshHandle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    // Advanced Sync - Peer-to-Peer Sync Methods
    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_create")]
    public static int P2PCreate(ulong localTreeHandle, ulong remoteTreeHandle, IntPtr outP2PPtr)
    {
        try {
            var localTree = Trees.Get(localTreeHandle);
            var remoteTree = Trees.Get(remoteTreeHandle);
            var p2p = AcornFacade.CreateP2P(localTree, remoteTree);
            ulong p2pHandle = P2PConnections.Add(p2p);
            unsafe { *(ulong*)outP2PPtr = p2pHandle; }
            return 0;
        } catch (Exception ex) {
            unsafe { *(ulong*)outP2PPtr = 0; }
            Error.Set(ex);
            return -1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_sync_bidirectional")]
    public static int P2PSyncBidirectional(ulong p2pHandle)
    {
        try {
            var p2p = P2PConnections.Get(p2pHandle);
            p2p.SyncBidirectional();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_sync_push_only")]
    public static int P2PSyncPushOnly(ulong p2pHandle)
    {
        try {
            var p2p = P2PConnections.Get(p2pHandle);
            p2p.SyncPushOnly();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_sync_pull_only")]
    public static int P2PSyncPullOnly(ulong p2pHandle)
    {
        try {
            var p2p = P2PConnections.Get(p2pHandle);
            p2p.SyncPullOnly();
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_set_sync_mode")]
    public static int P2PSetSyncMode(ulong p2pHandle, int syncMode)
    {
        try {
            var p2p = P2PConnections.Get(p2pHandle);
            p2p.SetSyncMode(syncMode);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_set_conflict_direction")]
    public static int P2PSetConflictDirection(ulong p2pHandle, int conflictDirection)
    {
        try {
            var p2p = P2PConnections.Get(p2pHandle);
            p2p.SetConflictDirection(conflictDirection);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_p2p_close")]
    public static int P2PClose(ulong p2pHandle)
    {
        try {
            P2PConnections.Remove(p2pHandle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    // Encryption support
    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_from_password")]
    public static int EncryptionFromPassword(IntPtr passwordUtf8, IntPtr saltUtf8, IntPtr handlePtr)
    {
        try {
            string password = Utf8.In(passwordUtf8);
            string salt = Utf8.In(saltUtf8);
            var encryption = AcornFacade.CreateEncryptionFromPassword(password, salt);
            ulong handle = EncryptionProviders.Add(encryption);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_from_key_iv")]
    public static int EncryptionFromKeyIV(IntPtr keyBase64Utf8, IntPtr ivBase64Utf8, IntPtr handlePtr)
    {
        try {
            string keyBase64 = Utf8.In(keyBase64Utf8);
            string ivBase64 = Utf8.In(ivBase64Utf8);
            var encryption = AcornFacade.CreateEncryptionFromKeyIV(keyBase64, ivBase64);
            ulong handle = EncryptionProviders.Add(encryption);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_generate_key_iv")]
    public static int EncryptionGenerateKeyIV(IntPtr keyBufPtr, IntPtr ivBufPtr)
    {
        try {
            var (keyBase64, ivBase64) = AcornFacade.GenerateKeyAndIV();
            
            // Allocate and return key
            var keyBytes = System.Text.Encoding.UTF8.GetBytes(keyBase64);
            unsafe {
                var keyBuf = (AcornBuf*)keyBufPtr;
                keyBuf->data = (byte*)Marshal.AllocHGlobal(keyBytes.Length);
                Marshal.Copy(keyBytes, 0, (IntPtr)keyBuf->data, keyBytes.Length);
                keyBuf->len = (nuint)keyBytes.Length;
            }
            
            // Allocate and return IV
            var ivBytes = System.Text.Encoding.UTF8.GetBytes(ivBase64);
            unsafe {
                var ivBuf = (AcornBuf*)ivBufPtr;
                ivBuf->data = (byte*)Marshal.AllocHGlobal(ivBytes.Length);
                Marshal.Copy(ivBytes, 0, (IntPtr)ivBuf->data, ivBytes.Length);
                ivBuf->len = (nuint)ivBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_export_key")]
    public static int EncryptionExportKey(ulong encryptionHandle, IntPtr keyBufPtr)
    {
        try {
            var encryption = EncryptionProviders.Get(encryptionHandle);
            var keyBase64 = encryption.ExportKeyBase64();
            var keyBytes = System.Text.Encoding.UTF8.GetBytes(keyBase64);
            
            unsafe {
                var keyBuf = (AcornBuf*)keyBufPtr;
                keyBuf->data = (byte*)Marshal.AllocHGlobal(keyBytes.Length);
                Marshal.Copy(keyBytes, 0, (IntPtr)keyBuf->data, keyBytes.Length);
                keyBuf->len = (nuint)keyBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_export_iv")]
    public static int EncryptionExportIV(ulong encryptionHandle, IntPtr ivBufPtr)
    {
        try {
            var encryption = EncryptionProviders.Get(encryptionHandle);
            var ivBase64 = encryption.ExportIVBase64();
            var ivBytes = System.Text.Encoding.UTF8.GetBytes(ivBase64);
            
            unsafe {
                var ivBuf = (AcornBuf*)ivBufPtr;
                ivBuf->data = (byte*)Marshal.AllocHGlobal(ivBytes.Length);
                Marshal.Copy(ivBytes, 0, (IntPtr)ivBuf->data, ivBytes.Length);
                ivBuf->len = (nuint)ivBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_encrypt")]
    public static int EncryptionEncrypt(ulong encryptionHandle, IntPtr plaintextUtf8, IntPtr ciphertextBufPtr)
    {
        try {
            var encryption = EncryptionProviders.Get(encryptionHandle);
            string plaintext = Utf8.In(plaintextUtf8);
            var ciphertext = encryption.Encrypt(plaintext);
            var ciphertextBytes = System.Text.Encoding.UTF8.GetBytes(ciphertext);
            
            unsafe {
                var ciphertextBuf = (AcornBuf*)ciphertextBufPtr;
                ciphertextBuf->data = (byte*)Marshal.AllocHGlobal(ciphertextBytes.Length);
                Marshal.Copy(ciphertextBytes, 0, (IntPtr)ciphertextBuf->data, ciphertextBytes.Length);
                ciphertextBuf->len = (nuint)ciphertextBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_decrypt")]
    public static int EncryptionDecrypt(ulong encryptionHandle, IntPtr ciphertextUtf8, IntPtr plaintextBufPtr)
    {
        try {
            var encryption = EncryptionProviders.Get(encryptionHandle);
            string ciphertext = Utf8.In(ciphertextUtf8);
            var plaintext = encryption.Decrypt(ciphertext);
            var plaintextBytes = System.Text.Encoding.UTF8.GetBytes(plaintext);
            
            unsafe {
                var plaintextBuf = (AcornBuf*)plaintextBufPtr;
                plaintextBuf->data = (byte*)Marshal.AllocHGlobal(plaintextBytes.Length);
                Marshal.Copy(plaintextBytes, 0, (IntPtr)plaintextBuf->data, plaintextBytes.Length);
                plaintextBuf->len = (nuint)plaintextBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_is_enabled")]
    public static int EncryptionIsEnabled(ulong encryptionHandle)
    {
        try {
            var encryption = EncryptionProviders.Get(encryptionHandle);
            return encryption.IsEnabled ? 1 : 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_encryption_close")]
    public static int EncryptionClose(ulong encryptionHandle)
    {
        try {
            EncryptionProviders.Remove(encryptionHandle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_open_tree_encrypted")]
    public static int OpenTreeEncrypted(IntPtr uriUtf8, ulong encryptionHandle, IntPtr handlePtr)
    {
        try {
            string uri = Utf8.In(uriUtf8);
            var encryption = EncryptionProviders.Get(encryptionHandle);
            var tree = AcornFacade.OpenJsonTreeEncrypted(uri, encryption);
            ulong handle = Trees.Add(tree);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_open_tree_encrypted_compressed")]
    public static int OpenTreeEncryptedCompressed(IntPtr uriUtf8, ulong encryptionHandle, int compressionLevel, IntPtr handlePtr)
    {
        try {
            string uri = Utf8.In(uriUtf8);
            var encryption = EncryptionProviders.Get(encryptionHandle);
            var compressionLevelEnum = compressionLevel switch {
                0 => System.IO.Compression.CompressionLevel.Fastest,
                1 => System.IO.Compression.CompressionLevel.Optimal,
                2 => System.IO.Compression.CompressionLevel.SmallestSize,
                _ => System.IO.Compression.CompressionLevel.Optimal
            };
            var tree = AcornFacade.OpenJsonTreeEncryptedCompressed(uri, encryption, compressionLevelEnum);
            ulong handle = Trees.Add(tree);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    // Compression FFI functions
    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_gzip")]
    public static int CompressionGzip(int compressionLevel, IntPtr handlePtr)
    {
        try {
            var compressionLevelEnum = compressionLevel switch {
                0 => System.IO.Compression.CompressionLevel.Fastest,
                1 => System.IO.Compression.CompressionLevel.Optimal,
                2 => System.IO.Compression.CompressionLevel.SmallestSize,
                _ => System.IO.Compression.CompressionLevel.Optimal
            };
            var compression = AcornFacade.CreateGzipCompression(compressionLevelEnum);
            ulong handle = CompressionProviders.Add(compression);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_brotli")]
    public static int CompressionBrotli(int compressionLevel, IntPtr handlePtr)
    {
        try {
            var compressionLevelEnum = compressionLevel switch {
                0 => System.IO.Compression.CompressionLevel.Fastest,
                1 => System.IO.Compression.CompressionLevel.Optimal,
                2 => System.IO.Compression.CompressionLevel.SmallestSize,
                _ => System.IO.Compression.CompressionLevel.Optimal
            };
            var compression = AcornFacade.CreateBrotliCompression(compressionLevelEnum);
            ulong handle = CompressionProviders.Add(compression);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_none")]
    public static int CompressionNone(IntPtr handlePtr)
    {
        try {
            var compression = AcornFacade.CreateNoCompression();
            ulong handle = CompressionProviders.Add(compression);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_compress")]
    public static int CompressionCompress(ulong compressionHandle, IntPtr dataUtf8, IntPtr compressedBufPtr)
    {
        try {
            var compression = CompressionProviders.Get(compressionHandle);
            string data = Utf8.In(dataUtf8);
            var compressed = compression.Compress(data);
            var compressedBytes = System.Text.Encoding.UTF8.GetBytes(compressed);
            
            unsafe {
                var compressedBuf = (AcornBuf*)compressedBufPtr;
                compressedBuf->data = (byte*)Marshal.AllocHGlobal(compressedBytes.Length);
                Marshal.Copy(compressedBytes, 0, (IntPtr)compressedBuf->data, compressedBytes.Length);
                compressedBuf->len = (nuint)compressedBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_decompress")]
    public static int CompressionDecompress(ulong compressionHandle, IntPtr compressedDataUtf8, IntPtr dataBufPtr)
    {
        try {
            var compression = CompressionProviders.Get(compressionHandle);
            string compressedData = Utf8.In(compressedDataUtf8);
            var data = compression.Decompress(compressedData);
            var dataBytes = System.Text.Encoding.UTF8.GetBytes(data);
            
            unsafe {
                var dataBuf = (AcornBuf*)dataBufPtr;
                dataBuf->data = (byte*)Marshal.AllocHGlobal(dataBytes.Length);
                Marshal.Copy(dataBytes, 0, (IntPtr)dataBuf->data, dataBytes.Length);
                dataBuf->len = (nuint)dataBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_is_enabled")]
    public static int CompressionIsEnabled(ulong compressionHandle)
    {
        try {
            var compression = CompressionProviders.Get(compressionHandle);
            return compression.IsEnabled ? 1 : 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_algorithm_name")]
    public static int CompressionAlgorithmName(ulong compressionHandle, IntPtr nameBufPtr)
    {
        try {
            var compression = CompressionProviders.Get(compressionHandle);
            var nameBytes = System.Text.Encoding.UTF8.GetBytes(compression.AlgorithmName);
            
            unsafe {
                var nameBuf = (AcornBuf*)nameBufPtr;
                nameBuf->data = (byte*)Marshal.AllocHGlobal(nameBytes.Length);
                Marshal.Copy(nameBytes, 0, (IntPtr)nameBuf->data, nameBytes.Length);
                nameBuf->len = (nuint)nameBytes.Length;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_get_stats")]
    public static int CompressionGetStats(ulong compressionHandle, IntPtr originalDataUtf8, IntPtr compressedDataUtf8, 
                                         IntPtr originalSizePtr, IntPtr compressedSizePtr, IntPtr ratioPtr, IntPtr spaceSavedPtr)
    {
        try {
            var compression = CompressionProviders.Get(compressionHandle);
            string originalData = Utf8.In(originalDataUtf8);
            string compressedData = Utf8.In(compressedDataUtf8);
            var stats = compression.GetStats(originalData, compressedData);
            
            unsafe {
                *(int*)originalSizePtr = stats.OriginalSize;
                *(int*)compressedSizePtr = stats.CompressedSize;
                *(double*)ratioPtr = stats.Ratio;
                *(int*)spaceSavedPtr = stats.SpaceSaved;
            }
            
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_compression_close")]
    public static int CompressionClose(ulong compressionHandle)
    {
        try {
            CompressionProviders.Remove(compressionHandle, out _);
            return 0;
        } catch (Exception ex) { Error.Set(ex); return -1; }
    }

    [UnmanagedCallersOnly(EntryPoint = "acorn_open_tree_compressed")]
    public static int OpenTreeCompressed(IntPtr uriUtf8, ulong compressionHandle, IntPtr handlePtr)
    {
        try {
            string uri = Utf8.In(uriUtf8);
            var compression = CompressionProviders.Get(compressionHandle);
            var tree = AcornFacade.OpenJsonTreeCompressed(uri, compression);
            ulong handle = Trees.Add(tree);
            unsafe { *(ulong*)handlePtr = handle; }
            return 0;
        } catch (Exception ex) { 
            unsafe { *(ulong*)handlePtr = 0; }
            Error.Set(ex); 
            return -1; 
        }
    }
}
