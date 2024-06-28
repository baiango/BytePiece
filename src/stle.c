// Standard Library Extended
#include "stle.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>


#define IMPL_MIN_MAX(type) \
type type##_min(type a, type b) { \
	return a < b ? a : b; \
} \
 \
type type##_max(type a, type b) { \
	return a > b ? a : b; \
} \
 \
mut_##type mut_##type##_min(mut_##type a, mut_##type b) { \
	return a < b ? a : b; \
} \
 \
mut_##type mut_##type##_max(mut_##type a, mut_##type b) { \
	return a > b ? a : b; \
}

IMPL_MIN_MAX(i8)
IMPL_MIN_MAX(i16)
IMPL_MIN_MAX(i32)
IMPL_MIN_MAX(i64)

IMPL_MIN_MAX(u8)
IMPL_MIN_MAX(u16)
IMPL_MIN_MAX(u32)
IMPL_MIN_MAX(u64)

IMPL_MIN_MAX(f32)
IMPL_MIN_MAX(f64)


TrieNode *stle_trie_new() {
	TrieNode *root = (TrieNode *)malloc(sizeof(TrieNode));

	if (root) {
		root->is_end_of_word = false;

		for (mut_u8 i = 0; i < UINT8_MAX; i++) {
			root->children[i] = EMPTY_TRIE;
		}
	}

	return root;
}

ErrorCode stle_trie_insert_key(TrieNode *current_node, i8 *key) {
	u32 key_len = strlen(key);

	for (mut_u8 depth = 0; depth < key_len; depth++) {
		u8 index = key[depth];

		if (!current_node->children[index]) {
			current_node->children[index] = stle_trie_new();
		}

		current_node = current_node->children[index];
	}

	current_node->is_end_of_word = true;
	return OK;
}

u1 stle_trie_search(TrieNode *current_node, i8 *key) {
	u32 length = strlen(key);

	for (mut_u8 depth = 0; depth < length; depth++) {
		i32 index = key[depth];

		if (!current_node->children[index]) {
			return false;
		}

		current_node = current_node->children[index];
	}

	return (EMPTY_TRIE != current_node && current_node->is_end_of_word);
}

ErrorCode stle_read_file(i8 file_path[], u32 bytes_to_read, mut_i8 **data) {
	FILE *file = fopen(file_path, "r");
	if (file == NULL) {
		return FILE_NOT_FOUND;
	}

	// Get the size of the file
	fseek(file, 0, SEEK_END);
	u32 file_size = u32_min(ftell(file), bytes_to_read);
	rewind(file);

	// Allocate enough memory to hold the contents of the file
	*data = malloc((file_size + 1) * sizeof(i8));
	if (*data == NULL) {
		fclose(file);
		return MALLOC_ERR;
	}

	// Copy the contents of the file into the buffer
	size_t num_read;
	size_t total_read = 0;
	while ((num_read = fread(*data + total_read, sizeof(i8), 1023, file)) > 0) {
		total_read += num_read;
	}

	// Ensure the buffer is properly null-terminated
	(*data)[total_read] = '\0';

	// Close the file
	fclose(file);

	return OK;
}

ErrorCode test_stle_trie() {
	TrieNode *root = stle_trie_new();

	stle_trie_insert_key(root, "hello");
	stle_trie_insert_key(root, "world");

	printf("Trie \"hello\": %s, ", stle_trie_search(root, "hello") ? "Found" : "Not Found");
	printf("Trie \"hell\": %s, ", stle_trie_search(root, "hell") ? "Found" : "Not Found");
	printf("Trie \"world\": %s, ", stle_trie_search(root, "world") ? "Found" : "Not Found");
	printf("Trie \"hi\": %s\n", stle_trie_search(root, "hi") ? "Found" : "Not Found");

	return OK;
}

ErrorCode test_stle_read_file() {
	i8 file_path[] = "src/main.c";
	mut_i8 *data;
	if (stle_read_file(file_path, 0xfff, &data) != OK) {
		printf("Failed to read file %s!\n", file_path);
		return FILE_NOT_FOUND;
	}

	printf("%s", data);
	free(data);
	return OK;
}

ErrorCode test_modules() {
	ErrorCode result = OK;
	result += test_stle_read_file();
	result += test_stle_trie();
	return result;
}
