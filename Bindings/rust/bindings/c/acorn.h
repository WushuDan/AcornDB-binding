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
