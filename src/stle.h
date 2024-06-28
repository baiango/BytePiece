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

DEF_DATA(bool, u1)

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
	FILE_NOT_FOUND,
	MALLOC_ERR,
} ErrorCode;

typedef struct TrieNode {
	struct TrieNode *children[UINT8_MAX];
	mut_u1 is_end_of_word;
} TrieNode;


TrieNode *stle_trie_new();
ErrorCode stle_trie_insert_key(TrieNode *crawl_node, i8 *key);
u1 stle_trie_search(TrieNode *crawl_node, i8 *key);
ErrorCode stle_read_file(i8 file_path[], u32 bytes_to_read, mut_i8 **data);
ErrorCode test_stle_trie();
ErrorCode test_stle_read_file();
ErrorCode test_modules();
