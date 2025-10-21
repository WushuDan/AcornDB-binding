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

// Reactive programming support
typedef uint64_t acorn_reactive_stream_handle;

// Change types
typedef enum {
    ACORN_CHANGE_STASH = 0,
    ACORN_CHANGE_TOSS = 1,
    ACORN_CHANGE_SQUABBLE = 2
} acorn_change_type;

// Reactive stream creation
ACORN_API int acorn_create_change_stream(acorn_tree_handle tree, acorn_reactive_stream_handle* out_stream);
ACORN_API int acorn_create_filtered_stream(acorn_tree_handle tree, acorn_change_type change_type, acorn_reactive_stream_handle* out_stream);
ACORN_API int acorn_create_buffered_stream(acorn_tree_handle tree, int buffer_ms, acorn_reactive_stream_handle* out_stream);
ACORN_API int acorn_create_throttled_stream(acorn_tree_handle tree, int throttle_ms, acorn_reactive_stream_handle* out_stream);
ACORN_API int acorn_create_sampled_stream(acorn_tree_handle tree, int sample_ms, acorn_reactive_stream_handle* out_stream);

// Reactive stream operations
ACORN_API int acorn_stream_subscribe(acorn_reactive_stream_handle stream, void (*callback)(const char* id, const char* json, size_t len, acorn_change_type change_type, void* user_data), void* user_data, acorn_sub_handle* out_subscription);
ACORN_API int acorn_stream_close(acorn_reactive_stream_handle stream);

// Git integration support
typedef uint64_t acorn_git_handle;

// Git commit information
typedef struct {
    char* sha;
    char* message;
    char* author;
    char* email;
    int64_t timestamp;
} acorn_git_commit_info;

// Git operations
ACORN_API int acorn_git_create(const char* repo_path, const char* author_name, const char* author_email, int auto_push, acorn_git_handle* out_git);
ACORN_API int acorn_git_push(acorn_git_handle git, const char* remote_name, const char* branch);
ACORN_API int acorn_git_pull(acorn_git_handle git, const char* remote_name, const char* branch);
ACORN_API int acorn_git_get_commit_log(acorn_git_handle git, const char* file_path, acorn_git_commit_info** out_commits, size_t* out_count);
ACORN_API int acorn_git_get_file_history(acorn_git_handle git, const char* file_path, acorn_git_commit_info** out_commits, size_t* out_count);
ACORN_API int acorn_git_read_file_at_commit(acorn_git_handle git, const char* file_path, const char* commit_sha, char** out_content, size_t* out_length);
ACORN_API int acorn_git_squash_commits(acorn_git_handle git, const char* since_commit);
ACORN_API int acorn_git_has_remote(acorn_git_handle git, const char* remote_name, int* out_has_remote);
ACORN_API int acorn_git_close(acorn_git_handle git);

// Free Git commit info array
ACORN_API void acorn_git_free_commit_info(acorn_git_commit_info* commits, size_t count);

// Nursery System - Dynamic trunk discovery and creation
typedef uint64_t acorn_nursery_handle;

// Trunk metadata information
typedef struct {
    char* type_id;
    char* display_name;
    char* description;
    char* category;
    int is_durable;
    int supports_history;
    int supports_sync;
    int supports_async;
    char** required_config_keys;
    size_t required_config_keys_count;
    char** optional_config_keys;
    size_t optional_config_keys_count;
    int is_built_in;
} acorn_trunk_metadata;

// Nursery operations
ACORN_API int acorn_nursery_create(acorn_nursery_handle* out_nursery);
ACORN_API int acorn_nursery_get_available_types(acorn_nursery_handle nursery, char*** out_types, size_t* out_count);
ACORN_API int acorn_nursery_get_metadata(acorn_nursery_handle nursery, const char* type_id, acorn_trunk_metadata* out_metadata);
ACORN_API int acorn_nursery_get_all_metadata(acorn_nursery_handle nursery, acorn_trunk_metadata** out_metadata, size_t* out_count);
ACORN_API int acorn_nursery_has_trunk(acorn_nursery_handle nursery, const char* type_id, int* out_has_trunk);
ACORN_API int acorn_nursery_grow_trunk(acorn_nursery_handle nursery, const char* type_id, const char* config_json, acorn_storage_handle* out_storage);
ACORN_API int acorn_nursery_validate_config(acorn_nursery_handle nursery, const char* type_id, const char* config_json, int* out_valid);
ACORN_API int acorn_nursery_get_catalog(acorn_nursery_handle nursery, char** out_catalog, size_t* out_length);
ACORN_API int acorn_nursery_close(acorn_nursery_handle nursery);

// Free nursery resources
ACORN_API void acorn_nursery_free_types(char** types, size_t count);
ACORN_API void acorn_nursery_free_metadata(acorn_trunk_metadata* metadata, size_t count);
ACORN_API void acorn_nursery_free_catalog(char* catalog);

// Advanced Tree Features - Auto-ID detection, TTL enforcement, statistics
typedef uint64_t acorn_tree_stats_handle;

// Tree statistics information
typedef struct {
    int total_stashed;
    int total_tossed;
    int squabbles_resolved;
    int smushes_performed;
    int active_tangles;
    int64_t last_sync_timestamp;
} acorn_tree_stats;

// TTL enforcement information
typedef struct {
    int ttl_enforcement_enabled;
    int64_t cleanup_interval_ms;
    int expiring_nuts_count;
} acorn_ttl_info;

// Advanced tree operations
ACORN_API int acorn_tree_stash_auto_id(acorn_tree_handle tree, const char* json, size_t len);
ACORN_API int acorn_tree_get_stats(acorn_tree_handle tree, acorn_tree_stats* out_stats);
ACORN_API int acorn_tree_get_ttl_info(acorn_tree_handle tree, acorn_ttl_info* out_ttl_info);
ACORN_API int acorn_tree_set_ttl_enforcement(acorn_tree_handle tree, int enabled);
ACORN_API int acorn_tree_set_cleanup_interval(acorn_tree_handle tree, int64_t interval_ms);
ACORN_API int acorn_tree_cleanup_expired_nuts(acorn_tree_handle tree, int* out_removed_count);
ACORN_API int acorn_tree_get_expiring_nuts_count(acorn_tree_handle tree, int64_t timespan_ms, int* out_count);
ACORN_API int acorn_tree_get_expiring_nuts(acorn_tree_handle tree, int64_t timespan_ms, char*** out_ids, size_t* out_count);
ACORN_API int acorn_tree_get_all_nuts(acorn_tree_handle tree, char** out_json, size_t* out_length);
ACORN_API int acorn_tree_get_nut_count(acorn_tree_handle tree, int* out_count);
ACORN_API int acorn_tree_get_last_sync_timestamp(acorn_tree_handle tree, int64_t* out_timestamp);

// Free advanced tree resources
ACORN_API void acorn_tree_free_expiring_nuts(char** ids, size_t count);
ACORN_API void acorn_tree_free_all_nuts(char* json);

// Event Management - Enhanced event system, tangle support, mesh primitives
typedef uint64_t acorn_event_manager_handle;
typedef uint64_t acorn_tangle_handle;
typedef uint64_t acorn_mesh_coordinator_handle;

// Event types
typedef enum {
    ACORN_EVENT_STASH = 0,
    ACORN_EVENT_TOSS = 1,
    ACORN_EVENT_SQUABBLE = 2,
    ACORN_EVENT_SYNC = 3
} acorn_event_type;

// Event information
typedef struct {
    acorn_event_type event_type;
    char* key;
    char* json_payload;
    size_t json_length;
    int64_t timestamp;
    char* source_node;
} acorn_event_info;

// Mesh topology types
typedef enum {
    ACORN_MESH_FULL = 0,
    ACORN_MESH_RING = 1,
    ACORN_MESH_STAR = 2,
    ACORN_MESH_CUSTOM = 3
} acorn_mesh_topology;

// Mesh statistics
typedef struct {
    char* node_id;
    int tracked_change_ids;
    int active_tangles;
    int max_hop_count;
    int total_sync_operations;
    int64_t last_sync_timestamp;
} acorn_mesh_stats;

// Enhanced event management
ACORN_API int acorn_event_manager_create(acorn_tree_handle tree, acorn_event_manager_handle* out_manager);
ACORN_API int acorn_event_manager_subscribe(acorn_event_manager_handle manager, acorn_event_cb cb, void* user, acorn_sub_handle* out_sub);
ACORN_API int acorn_event_manager_subscribe_filtered(acorn_event_manager_handle manager, acorn_event_type event_type, acorn_event_cb cb, void* user, acorn_sub_handle* out_sub);
ACORN_API int acorn_event_manager_raise_event(acorn_event_manager_handle manager, acorn_event_type event_type, const char* key, const char* json_payload, size_t json_length);
ACORN_API int acorn_event_manager_get_subscriber_count(acorn_event_manager_handle manager, int* out_count);
ACORN_API int acorn_event_manager_close(acorn_event_manager_handle manager);

// Tangle management
ACORN_API int acorn_tangle_create(acorn_tree_handle local_tree, acorn_tree_handle remote_tree, const char* tangle_name, acorn_tangle_handle* out_tangle);
ACORN_API int acorn_tangle_create_in_process(acorn_tree_handle local_tree, acorn_tree_handle remote_tree, const char* tangle_name, acorn_tangle_handle* out_tangle);
ACORN_API int acorn_tangle_push(acorn_tangle_handle tangle, const char* key, const char* json_payload, size_t json_length);
ACORN_API int acorn_tangle_pull(acorn_tangle_handle tangle);
ACORN_API int acorn_tangle_sync_bidirectional(acorn_tangle_handle tangle);
ACORN_API int acorn_tangle_get_stats(acorn_tangle_handle tangle, acorn_mesh_stats* out_stats);
ACORN_API int acorn_tangle_close(acorn_tangle_handle tangle);

// Mesh coordinator
ACORN_API int acorn_mesh_coordinator_create(acorn_mesh_coordinator_handle* out_coordinator);
ACORN_API int acorn_mesh_coordinator_add_node(acorn_mesh_coordinator_handle coordinator, const char* node_id, acorn_tree_handle tree);
ACORN_API int acorn_mesh_coordinator_connect_nodes(acorn_mesh_coordinator_handle coordinator, const char* node_a, const char* node_b);
ACORN_API int acorn_mesh_coordinator_create_topology(acorn_mesh_coordinator_handle coordinator, acorn_mesh_topology topology, const char* hub_node_id);
ACORN_API int acorn_mesh_coordinator_synchronize_all(acorn_mesh_coordinator_handle coordinator);
ACORN_API int acorn_mesh_coordinator_get_node_stats(acorn_mesh_coordinator_handle coordinator, const char* node_id, acorn_mesh_stats* out_stats);
ACORN_API int acorn_mesh_coordinator_get_all_stats(acorn_mesh_coordinator_handle coordinator, acorn_mesh_stats** out_stats, size_t* out_count);
ACORN_API int acorn_mesh_coordinator_close(acorn_mesh_coordinator_handle coordinator);

// Free event management resources
ACORN_API void acorn_event_manager_free_event_info(acorn_event_info* event_info);
ACORN_API void acorn_mesh_coordinator_free_stats(acorn_mesh_stats* stats, size_t count);

// Performance Monitoring - Built-in metrics collection, health checks, monitoring
typedef uint64_t acorn_performance_monitor_handle;
typedef uint64_t acorn_health_checker_handle;

// Performance metrics
typedef struct {
    int64_t operations_per_second;
    int64_t memory_usage_bytes;
    int64_t cache_hit_rate_percent;
    int64_t sync_latency_ms;
    int64_t disk_io_bytes;
    int64_t network_bytes;
    int64_t cpu_usage_percent;
    int64_t timestamp;
} acorn_performance_metrics;

// Health check status
typedef enum {
    ACORN_HEALTH_UNKNOWN = 0,
    ACORN_HEALTH_HEALTHY = 1,
    ACORN_HEALTH_DEGRADED = 2,
    ACORN_HEALTH_UNHEALTHY = 3
} acorn_health_status;

// Health check information
typedef struct {
    acorn_health_status status;
    char* service_name;
    char* message;
    int64_t response_time_ms;
    int64_t timestamp;
    char* details;
} acorn_health_info;

// Benchmark configuration
typedef struct {
    int operation_count;
    int warmup_iterations;
    int measurement_iterations;
    int64_t timeout_ms;
    int enable_memory_tracking;
    int enable_disk_tracking;
    int enable_network_tracking;
} acorn_benchmark_config;

// Benchmark results
typedef struct {
    char* operation_name;
    int64_t total_time_ms;
    int64_t operations_per_second;
    int64_t memory_allocated_bytes;
    int64_t disk_io_bytes;
    int64_t network_bytes;
    double average_latency_ms;
    double p50_latency_ms;
    double p95_latency_ms;
    double p99_latency_ms;
    int64_t timestamp;
} acorn_benchmark_result;

// Performance monitoring
ACORN_API int acorn_performance_monitor_create(acorn_performance_monitor_handle* out_monitor);
ACORN_API int acorn_performance_monitor_start_collection(acorn_performance_monitor_handle monitor);
ACORN_API int acorn_performance_monitor_stop_collection(acorn_performance_monitor_handle monitor);
ACORN_API int acorn_performance_monitor_get_metrics(acorn_performance_monitor_handle monitor, acorn_performance_metrics* out_metrics);
ACORN_API int acorn_performance_monitor_get_history(acorn_performance_monitor_handle monitor, acorn_performance_metrics** out_metrics, size_t* out_count);
ACORN_API int acorn_performance_monitor_reset_metrics(acorn_performance_monitor_handle monitor);
ACORN_API int acorn_performance_monitor_close(acorn_performance_monitor_handle monitor);

// Health checking
ACORN_API int acorn_health_checker_create(acorn_health_checker_handle* out_checker);
ACORN_API int acorn_health_checker_add_service(acorn_health_checker_handle checker, const char* service_name, const char* health_endpoint);
ACORN_API int acorn_health_checker_check_all(acorn_health_checker_handle checker, acorn_health_info** out_results, size_t* out_count);
ACORN_API int acorn_health_checker_check_service(acorn_health_checker_handle checker, const char* service_name, acorn_health_info* out_result);
ACORN_API int acorn_health_checker_get_overall_status(acorn_health_checker_handle checker, acorn_health_status* out_status);
ACORN_API int acorn_health_checker_close(acorn_health_checker_handle checker);

// Benchmarking
ACORN_API int acorn_benchmark_tree_operations(acorn_tree_handle tree, acorn_benchmark_config* config, acorn_benchmark_result** out_results, size_t* out_count);
ACORN_API int acorn_benchmark_sync_operations(acorn_tangle_handle tangle, acorn_benchmark_config* config, acorn_benchmark_result** out_results, size_t* out_count);
ACORN_API int acorn_benchmark_mesh_operations(acorn_mesh_coordinator_handle coordinator, acorn_benchmark_config* config, acorn_benchmark_result** out_results, size_t* out_count);

// Resource monitoring
ACORN_API int acorn_get_memory_usage(int64_t* out_heap_bytes, int64_t* out_stack_bytes, int64_t* out_total_bytes);
ACORN_API int acorn_get_disk_usage(const char* path, int64_t* out_used_bytes, int64_t* out_total_bytes, int64_t* out_free_bytes);
ACORN_API int acorn_get_system_info(char** out_info, size_t* out_length);

// Free performance monitoring resources
ACORN_API void acorn_performance_monitor_free_metrics(acorn_performance_metrics* metrics, size_t count);
ACORN_API void acorn_health_checker_free_results(acorn_health_info* results, size_t count);
ACORN_API void acorn_benchmark_free_results(acorn_benchmark_result* results, size_t count);
ACORN_API void acorn_free_system_info(char* info);

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
