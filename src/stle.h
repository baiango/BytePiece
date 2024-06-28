#pragma once
#include <stdbool.h>
#include <stdint.h>


typedef const bool u1;
typedef bool mut_u1;

typedef const char i8;
typedef const short i16;
typedef const int i32;
typedef const long long i64;
typedef char mut_i8;
typedef short mut_i16;
typedef int mut_i32;
typedef long long mut_i64;

typedef const unsigned char u8;
typedef const unsigned short u16;
typedef const unsigned int u32;
typedef const unsigned long long u64;
typedef unsigned char mut_u8;
typedef unsigned short mut_u16;
typedef unsigned int mut_u32;
typedef unsigned long long mut_u64;

typedef const float f32;
typedef const double f64;
typedef float mut_f32;
typedef double mut_f64;

#define EMPTY_TRIE NULL


typedef struct TrieNode {
	struct TrieNode *children[UINT8_MAX];
	mut_u1 is_end_of_word;
} TrieNode;


TrieNode *stle_trie_new();
void stle_trie_insert_key(TrieNode *crawl_node, i8 *key);
u1 stle_trie_search(TrieNode *crawl_node, i8 *key);
i32 stle_read_file(i8 file_path[], mut_i8 **data);
i32 test_stle_trie();
i32 test_stle_read_file();
i32 test_modules();
