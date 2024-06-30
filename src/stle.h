#pragma once
#include <stdbool.h>
#include <stdint.h>


#define TYPEDEF(T, N) \
typedef const T N; \
typedef T mut_##N;

#define DEF_MIN_MAX(T) \
T T##_min(T a, T b); \
T T##_max(T a, T b); \
mut_##T mut_##T##_min(mut_##T a, mut_##T b); \
mut_##T mut_##T##_max(mut_##T a, mut_##T b);

#define DEF_DATA(T, N) \
TYPEDEF(T, N) \
DEF_MIN_MAX(N)

TYPEDEF(bool, u1)

DEF_DATA(char, i8)
DEF_DATA(short, i16)
DEF_DATA(int, i32)
DEF_DATA(long long, i64)

DEF_DATA(unsigned char, u8)
DEF_DATA(unsigned short, u16)
DEF_DATA(unsigned int, u32)
DEF_DATA(unsigned long long, u64)

DEF_DATA(float, f32)
DEF_DATA(double, f64)


#define EMPTY_TRIE NULL

typedef enum ErrorCode {
	OK,
	FILE_NOT_FOUND_ERR,
	MALLOC_ERR,
	TRIE_NODE_NOT_FOUND_ERR,
	EMPTY_ERR,
} ErrorCode;

typedef struct TrieNode {
	struct TrieNode *children[UINT8_MAX];
	mut_u32 value;
	mut_u1 is_end_of_word;
} TrieNode;

typedef struct Bytes {
	mut_u32 len;
	mut_u8 *data;
} Bytes;


TrieNode *stle_trie_new();
void stle_print_all_keys_and_values_unchecked(const TrieNode* node, u8 *prefix, u32 depth);
ErrorCode stle_trie_prt_all_unchecked(const TrieNode* root);
ErrorCode stle_trie_insert_unchecked(TrieNode *crawl_node, Bytes *key);
u1 stle_trie_search_unchecked(TrieNode *crawl_node, Bytes *key);
ErrorCode stle_trie_get_unchecked(TrieNode **return_node, Bytes *key);
ErrorCode stle_read_file(u8 file_path[], u32 bytes_to_read, mut_u8 **data);
ErrorCode stle_test_trie();
ErrorCode stle_test_read_file();
ErrorCode stle_test_modules();
