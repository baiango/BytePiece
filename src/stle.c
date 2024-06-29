// Standard Library Extended
#include "stle.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>


#define IMPL_MIN_MAX(type) \
type type##_min(type a, type b) { return a < b ? a : b; } \
type type##_max(type a, type b) { return a > b ? a : b; } \
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
		root->value = 0;

		for (mut_u8 i = 0; i < UINT8_MAX; i++) {
			root->children[i] = EMPTY_TRIE;
		}
	}

	return root;
}


// Function to print out the contents of a Trie node
void stle_print_all_keys_and_values_unchecked(const TrieNode* node, u8 *prefix, u32 depth) {
	u8 MAX_PRINT_DEPTH = 32;

	if (!node) {
		return;
	}
	// Print the current character in the prefix
	if (node->is_end_of_word) {
		printf("[");
		for (mut_u32 i = 0; i < u32_min(depth - 1, MAX_PRINT_DEPTH - 1); ++i) {
			printf("%x,", prefix[i]);
		}
		printf("%x", prefix[depth - 1]);
		if (depth > MAX_PRINT_DEPTH) {
			printf("...");
		}
		printf("]    %u\n", node->value);
	}

	// Recursively traverse each child node
	for (mut_u8 c = 0; c < UINT8_MAX; ++c) {
		if (node->children[c]) {
			mut_u8 next_prefix[depth + 1];
			strcpy((mut_i8 *)next_prefix, (i8 *)prefix);
			next_prefix[depth] = c;
			stle_print_all_keys_and_values(node->children[c], next_prefix, depth + 1);
		}
	}
}

// Caller function to initialize the root node and pass it to the printing function
ErrorCode stle_trie_prt_all(const TrieNode* root) {
	stle_print_all_keys_and_values(root, (u8 *)"", 0);
	return OK;
}

ErrorCode stle_trie_insert_unchecked(TrieNode *search_node, Bytes *key) {
	// Don't put any checks in here! It should be done by the caller!
	for (mut_u32 depth = 0; depth < key->len; depth++) {
		u8 index = key->data[depth];

		if (!search_node->children[index]) {
			search_node->children[index] = stle_trie_new();
		}
		search_node = search_node->children[index];
	}

	search_node->is_end_of_word = true;
	return OK;
}

u1 stle_trie_search_unchecked(TrieNode *search_node, Bytes *key) {
	// Don't put any checks in here! It should be done by the caller!
	for (mut_u32 depth = 0; depth < key->len; depth++) {
		u8 index = key->data[depth];

		if (!search_node->children[index]) {
			return false;
		}

		search_node = search_node->children[index];
	}

	return (EMPTY_TRIE != search_node && search_node->is_end_of_word);
}

ErrorCode stle_trie_get_unchecked(TrieNode **return_node, Bytes *key) {
	// Don't put any checks in here! It should be done by the caller!
	for (mut_u32 depth = 0; depth < key->len; depth++) {
		u8 index = key->data[depth];

		if (!(*return_node)->children[index]) {
			return TRIE_NODE_NOT_FOUND_ERR;
		}

		*return_node = (*return_node)->children[index];
	}

	return OK;
}

ErrorCode stle_read_file(u8 file_path[], u32 bytes_to_read, mut_u8 **data) {
	FILE *file = fopen((i8 *)file_path, "r");
	if (file == NULL) {
		return FILE_NOT_FOUND_ERR;
	}

	// Get the size of the file
	fseek(file, 0, SEEK_END);
	u32 file_size = u32_min(ftell(file), bytes_to_read);
	rewind(file);

	// Allocate enough memory to hold the contents of the file
	*data = malloc((file_size + 1) * sizeof(u8));
	if (*data == NULL) {
		fclose(file);
		return MALLOC_ERR;
	}

	// Copy the contents of the file into the buffer
	size_t num_read;
	size_t total_read = 0;
	while ((num_read = fread(*data + total_read, sizeof(u8), 1023, file)) > 0) {
		total_read += num_read;
	}

	// Ensure the buffer is properly null-terminated
	(*data)[total_read] = '\0';

	// Close the file
	fclose(file);

	return OK;
}

ErrorCode stle_test_trie() {
	TrieNode *root = stle_trie_new();

	Bytes test_var_1 = {.data = (mut_u8 *)"hello", .len = 5};
	Bytes test_var_2 = {.data = (mut_u8 *)"world", .len = 5};
	stle_trie_insert_unchecked(root, &test_var_1);
	stle_trie_insert_unchecked(root, &test_var_2);


	TrieNode *child_node = root;
	u32 result = stle_trie_get_unchecked	(&child_node, &test_var_1);
	if (result) {
		printf("result is not ok: %u\n", result);
	}
	child_node->value += 3;


	printf("Trie \"hello\": %s, ", stle_trie_search_unchecked(root, &test_var_1) ? "Found" : "Not Found");
	printf("Trie \"hell\": %s, ", stle_trie_search_unchecked(root, &(Bytes){.data = (mut_u8 *)"hell", .len = 4}) ? "Found" : "Not Found");
	printf("Trie \"world\": %s, ", stle_trie_search_unchecked(root, &test_var_2) ? "Found" : "Not Found");
	printf("Trie \"hi\": %s\n", stle_trie_search_unchecked(root, &(Bytes){.data = (mut_u8 *)"hi", .len = 2}) ? "Found" : "Not Found");

	stle_trie_prt_all(root);
	return OK;
}

ErrorCode stle_test_read_file() {
	u8 file_path[] = "src/main.c";
	mut_u8 *data;
	if (stle_read_file(file_path, 0xfff, &data) != OK) {
		printf("Failed to read file %s!\n", file_path);
		return FILE_NOT_FOUND_ERR;
	}

	printf("%s", data);
	free(data);
	return OK;
}

ErrorCode stle_test_modules() {
	ErrorCode result = OK;
	result += stle_test_read_file();
	result += stle_test_trie();
	return result;
}
