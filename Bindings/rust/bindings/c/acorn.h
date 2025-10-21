#ifndef ACORN_H
#define ACORN_H

#include <stddef.h>
#include <stdint.h>

#ifdef _WIN32
  #define ACORN_API __declspec(dllexport)
#else
  #define ACORN_API __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
extern "C" {
#endif

// Error handling: Functions return 0 on success, 1 on not-found (where applicable), -1 on error.
// Call acorn_error_message() to retrieve a thread-local error string.

// Opaque handles
typedef uint64_t acorn_tree_handle;
typedef uint64_t acorn_iter_handle;
typedef uint64_t acorn_sub_handle;

// Owned buffer coming from shim -> caller. Caller must call acorn_free_buf.
typedef struct {
  uint8_t* data;
  size_t   len;
} acorn_buf;

// Open/Close
ACORN_API int acorn_open_tree(const char* storage_uri, acorn_tree_handle* out_tree);
ACORN_API int acorn_close_tree(acorn_tree_handle tree);

// CRUD (JSON bytes)
ACORN_API int acorn_stash_json(acorn_tree_handle tree,
                               const char* id,
                               const uint8_t* json,
                               size_t len);

ACORN_API int acorn_crack_json(acorn_tree_handle tree,
                               const char* id,
                               acorn_buf* out_json);

ACORN_API int acorn_delete(acorn_tree_handle tree, const char* id);

// Additional utility functions
ACORN_API int acorn_exists(acorn_tree_handle tree, const char* id);
ACORN_API int acorn_count(acorn_tree_handle tree, size_t* out_count);

// Iteration (point-in-time scan; iterator owns snapshot on shim side)
ACORN_API int acorn_iter_start(acorn_tree_handle tree, const char* prefix, acorn_iter_handle* out_iter);
ACORN_API int acorn_iter_next(acorn_iter_handle iter, acorn_buf* out_key, acorn_buf* out_json, int* out_done);
ACORN_API int acorn_iter_close(acorn_iter_handle iter);

// Subscriptions (callback invoked from a background thread in the shim)
typedef void (*acorn_event_cb)(const char* key,
                               const uint8_t* json,
                               size_t len,
                               void* user);

ACORN_API int acorn_subscribe(acorn_tree_handle tree, acorn_event_cb cb, void* user, acorn_sub_handle* out_sub);
ACORN_API int acorn_unsubscribe(acorn_sub_handle sub);

// Sync (optional)
ACORN_API int acorn_sync_http(acorn_tree_handle tree, const char* url);

// Advanced Sync - Mesh and Peer-to-Peer synchronization
typedef uint64_t acorn_mesh_handle;
typedef uint64_t acorn_p2p_handle;

// Mesh Sync - Coordinate synchronization across multiple trees
ACORN_API int acorn_mesh_create(acorn_mesh_handle* out_mesh);
ACORN_API int acorn_mesh_add_node(acorn_mesh_handle mesh, const char* node_id, acorn_tree_handle tree);
ACORN_API int acorn_mesh_connect_nodes(acorn_mesh_handle mesh, const char* node_a, const char* node_b);
ACORN_API int acorn_mesh_create_full_mesh(acorn_mesh_handle mesh);
ACORN_API int acorn_mesh_create_ring(acorn_mesh_handle mesh);
ACORN_API int acorn_mesh_create_star(acorn_mesh_handle mesh, const char* hub_node_id);
ACORN_API int acorn_mesh_synchronize_all(acorn_mesh_handle mesh);
ACORN_API int acorn_mesh_close(acorn_mesh_handle mesh);

// Peer-to-Peer Sync - Direct tree-to-tree synchronization
ACORN_API int acorn_p2p_create(acorn_tree_handle local_tree, acorn_tree_handle remote_tree, acorn_p2p_handle* out_p2p);
ACORN_API int acorn_p2p_sync_bidirectional(acorn_p2p_handle p2p);
ACORN_API int acorn_p2p_sync_push_only(acorn_p2p_handle p2p);
ACORN_API int acorn_p2p_sync_pull_only(acorn_p2p_handle p2p);
ACORN_API int acorn_p2p_set_sync_mode(acorn_p2p_handle p2p, int sync_mode); // 0=Bidirectional, 1=PushOnly, 2=PullOnly, 3=Disabled
ACORN_API int acorn_p2p_set_conflict_direction(acorn_p2p_handle p2p, int conflict_direction); // 0=UseJudge, 1=PreferLocal, 2=PreferRemote
ACORN_API int acorn_p2p_close(acorn_p2p_handle p2p);

// Transactions
typedef uint64_t acorn_transaction_handle;

ACORN_API int acorn_begin_transaction(acorn_tree_handle tree, acorn_transaction_handle* out_transaction);
ACORN_API int acorn_transaction_stash(acorn_transaction_handle transaction, const char* id, const uint8_t* json, size_t len);
ACORN_API int acorn_transaction_delete(acorn_transaction_handle transaction, const char* id);
ACORN_API int acorn_transaction_commit(acorn_transaction_handle transaction);
ACORN_API int acorn_transaction_rollback(acorn_transaction_handle transaction);
ACORN_API int acorn_transaction_close(acorn_transaction_handle transaction);

// Batch operations for improved performance when working with multiple items
// All batch operations return 0 on success, -1 on error

// Batch stash: Store multiple key-value pairs
// ids: array of null-terminated UTF-8 strings (keys)
// jsons: array of JSON byte buffers
// json_lens: array of JSON buffer lengths
// count: number of items to store
ACORN_API int acorn_batch_stash(acorn_tree_handle tree,
                                 const char** ids,
                                 const uint8_t** jsons,
                                 const size_t* json_lens,
                                 size_t count);

// Batch crack: Retrieve multiple values by their IDs
// ids: array of null-terminated UTF-8 strings (keys to retrieve)
// count: number of items to retrieve
// out_jsons: array of acorn_buf to receive JSON data (caller must free each with acorn_free_buf)
// out_found: array of int flags (1 if found, 0 if not found)
ACORN_API int acorn_batch_crack(acorn_tree_handle tree,
                                 const char** ids,
                                 size_t count,
                                 acorn_buf* out_jsons,
                                 int* out_found);

// Batch delete: Delete multiple items by their IDs
// ids: array of null-terminated UTF-8 strings (keys to delete)
// count: number of items to delete
ACORN_API int acorn_batch_delete(acorn_tree_handle tree,
                                  const char** ids,
                                  size_t count);

// Encryption support
typedef uint64_t acorn_encryption_handle;

// Compression support
typedef uint64_t acorn_compression_handle;

// Cache support
typedef uint64_t acorn_cache_handle;

// Conflict resolution support
typedef uint64_t acorn_conflict_judge_handle;

// Storage backend support
typedef uint64_t acorn_storage_handle;

// Document store support
typedef uint64_t acorn_document_store_handle;

// Create encryption provider from password
ACORN_API int acorn_encryption_from_password(const char* password, const char* salt, acorn_encryption_handle* out_encryption);

// Create encryption provider from explicit key/IV (base64 encoded)
ACORN_API int acorn_encryption_from_key_iv(const char* key_base64, const char* iv_base64, acorn_encryption_handle* out_encryption);

// Generate random key and IV (for testing/new deployments)
ACORN_API int acorn_encryption_generate_key_iv(acorn_buf* out_key_base64, acorn_buf* out_iv_base64);

// Export key/IV as base64 (for backup/storage)
ACORN_API int acorn_encryption_export_key(acorn_encryption_handle encryption, acorn_buf* out_key_base64);
ACORN_API int acorn_encryption_export_iv(acorn_encryption_handle encryption, acorn_buf* out_iv_base64);

// Encrypt/decrypt data
ACORN_API int acorn_encryption_encrypt(acorn_encryption_handle encryption, const char* plaintext, acorn_buf* out_ciphertext);
ACORN_API int acorn_encryption_decrypt(acorn_encryption_handle encryption, const char* ciphertext, acorn_buf* out_plaintext);

// Check if encryption is enabled
ACORN_API int acorn_encryption_is_enabled(acorn_encryption_handle encryption);

// Close encryption handle
ACORN_API int acorn_encryption_close(acorn_encryption_handle encryption);

// Open tree with encryption
ACORN_API int acorn_open_tree_encrypted(const char* storage_uri, acorn_encryption_handle encryption, acorn_tree_handle* out_tree);

// Open tree with encryption and compression
ACORN_API int acorn_open_tree_encrypted_compressed(const char* storage_uri, acorn_encryption_handle encryption, int compression_level, acorn_tree_handle* out_tree);

// Compression provider creation
ACORN_API int acorn_compression_gzip(int compression_level, acorn_compression_handle* out_compression);
ACORN_API int acorn_compression_brotli(int compression_level, acorn_compression_handle* out_compression);
ACORN_API int acorn_compression_none(acorn_compression_handle* out_compression);

// Compression operations
ACORN_API int acorn_compression_compress(acorn_compression_handle compression, const char* data, acorn_buf* out_compressed);
ACORN_API int acorn_compression_decompress(acorn_compression_handle compression, const char* compressed_data, acorn_buf* out_data);

// Compression info
ACORN_API int acorn_compression_is_enabled(acorn_compression_handle compression);
ACORN_API int acorn_compression_algorithm_name(acorn_compression_handle compression, acorn_buf* out_name);

// Compression statistics
ACORN_API int acorn_compression_get_stats(acorn_compression_handle compression, const char* original_data, const char* compressed_data, 
                                          int* out_original_size, int* out_compressed_size, double* out_ratio, int* out_space_saved);

// Close compression handle
ACORN_API int acorn_compression_close(acorn_compression_handle compression);

// Open tree with compression only
ACORN_API int acorn_open_tree_compressed(const char* storage_uri, acorn_compression_handle compression, acorn_tree_handle* out_tree);

// Cache strategy creation
ACORN_API int acorn_cache_lru(int max_size, acorn_cache_handle* out_cache);
ACORN_API int acorn_cache_no_eviction(acorn_cache_handle* out_cache);

// Cache operations
ACORN_API int acorn_cache_reset(acorn_cache_handle cache);
ACORN_API int acorn_cache_get_stats(acorn_cache_handle cache, int* out_tracked_items, int* out_max_size, double* out_utilization);

// Cache configuration
ACORN_API int acorn_cache_set_eviction_enabled(acorn_cache_handle cache, int enabled);
ACORN_API int acorn_cache_is_eviction_enabled(acorn_cache_handle cache);

// Close cache handle
ACORN_API int acorn_cache_close(acorn_cache_handle cache);

// Open tree with cache strategy
ACORN_API int acorn_open_tree_with_cache(const char* storage_uri, acorn_cache_handle cache, acorn_tree_handle* out_tree);

// Conflict resolution judge creation
ACORN_API int acorn_conflict_judge_timestamp(acorn_conflict_judge_handle* out_judge);
ACORN_API int acorn_conflict_judge_version(acorn_conflict_judge_handle* out_judge);
ACORN_API int acorn_conflict_judge_local_wins(acorn_conflict_judge_handle* out_judge);
ACORN_API int acorn_conflict_judge_remote_wins(acorn_conflict_judge_handle* out_judge);

// Conflict resolution operations
ACORN_API int acorn_conflict_judge_name(acorn_conflict_judge_handle judge, acorn_buf* out_name);
ACORN_API int acorn_conflict_judge_resolve(acorn_conflict_judge_handle judge, const char* local_json, const char* incoming_json, acorn_buf* out_winner_json);

// Close conflict judge handle
ACORN_API int acorn_conflict_judge_close(acorn_conflict_judge_handle judge);

// Open tree with conflict judge
ACORN_API int acorn_open_tree_with_conflict_judge(const char* storage_uri, acorn_conflict_judge_handle judge, acorn_tree_handle* out_tree);

// Storage backend creation
ACORN_API int acorn_storage_s3(const char* access_key, const char* secret_key, const char* bucket_name, const char* region, const char* prefix, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_s3_default(const char* bucket_name, const char* region, const char* prefix, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_s3_compatible(const char* access_key, const char* secret_key, const char* bucket_name, const char* service_url, const char* prefix, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_azure_blob(const char* connection_string, const char* container_name, const char* prefix, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_sqlite(const char* database_path, const char* table_name, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_postgresql(const char* connection_string, const char* table_name, const char* schema, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_mysql(const char* connection_string, const char* table_name, const char* database, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_sqlserver(const char* connection_string, const char* table_name, const char* schema, acorn_storage_handle* out_storage);
ACORN_API int acorn_storage_git(const char* repo_path, const char* author_name, const char* author_email, int auto_push, acorn_storage_handle* out_storage);

// Storage backend operations
ACORN_API int acorn_storage_get_info(acorn_storage_handle storage, acorn_buf* out_info);
ACORN_API int acorn_storage_test_connection(acorn_storage_handle storage);

// Close storage handle
ACORN_API int acorn_storage_close(acorn_storage_handle storage);

// Open tree with storage backend
ACORN_API int acorn_open_tree_with_storage(acorn_storage_handle storage, acorn_tree_handle* out_tree);

// Document store creation
ACORN_API int acorn_document_store_create(const char* custom_path, acorn_document_store_handle* out_document_store);

// Document store operations
ACORN_API int acorn_document_store_get_history(acorn_document_store_handle document_store, const char* id, acorn_buf* out_history_json);
ACORN_API int acorn_document_store_get_info(acorn_document_store_handle document_store, acorn_buf* out_info);
ACORN_API int acorn_document_store_compact(acorn_document_store_handle document_store);

// Close document store handle
ACORN_API int acorn_document_store_close(acorn_document_store_handle document_store);

// Open tree with document store
ACORN_API int acorn_open_tree_with_document_store(acorn_document_store_handle document_store, acorn_tree_handle* out_tree);

// Memory management for buffers allocated by shim
ACORN_API void acorn_free_buf(acorn_buf* buf);

// Last error (thread-local, null-terminated UTF-8 string). Pointer invalidated on next shim call.
// Caller must free the returned string with acorn_free_error_string().
ACORN_API const char* acorn_error_message(void);
ACORN_API void acorn_free_error_string(const char* str);

#ifdef __cplusplus
}
#endif

#endif // ACORN_H
