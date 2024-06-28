// Standard Library Extended
#include "stle.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>


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

void stle_trie_insert_key(TrieNode *crawl_node, i8 *key) {
	u32 key_len = strlen(key);

	for (mut_u8 level = 0; level < key_len; level++) {
		u8 index = key[level];

		if (!crawl_node->children[index]) {
			crawl_node->children[index] = stle_trie_new();
		}

		crawl_node = crawl_node->children[index];
	}

	crawl_node->is_end_of_word = true;
}

u1 stle_trie_search(TrieNode *crawl_node, i8 *key) {
	u32 length = strlen(key);

	for (mut_u8 level = 0; level < length; level++) {
		i32 index = key[level];

		if (!crawl_node->children[index]) {
			return false;
		}

		crawl_node = crawl_node->children[index];
	}

	return (EMPTY_TRIE != crawl_node && crawl_node->is_end_of_word);
}

i32 stle_read_file(i8 file_path[], mut_i8 **data) {
	FILE *file = fopen(file_path, "r");
	if (file == NULL) {
		return -1;
	}

	// Get the size of the file
	fseek(file, 0, SEEK_END);
	u32 file_size = ftell(file);
	rewind(file);

	// Allocate enough memory to hold the contents of the file
	*data = malloc((file_size + 1) * sizeof(i8));
	if (*data == NULL) {
		fclose(file);
		return -2;
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

	return 0;
}

i32 test_stle_trie() {
	TrieNode *root = stle_trie_new();

	stle_trie_insert_key(root, "hello");
	stle_trie_insert_key(root, "world");

	printf("Trie \"hello\": %s, ", stle_trie_search(root, "hello") ? "Found" : "Not Found");
	printf("Trie \"hell\": %s, ", stle_trie_search(root, "hell") ? "Found" : "Not Found");
	printf("Trie \"world\": %s, ", stle_trie_search(root, "world") ? "Found" : "Not Found");
	printf("Trie \"hi\": %s\n", stle_trie_search(root, "hi") ? "Found" : "Not Found");

	return 0;
}

i32 test_stle_read_file() {
	i8 file_path[] = "src/main.c";
	mut_i8 *data;
	if (stle_read_file(file_path, &data) != 0) {
		printf("Failed to read file %s!\n", file_path);
		return 1;
	}

	printf("%s", data);
	free(data);
	return 0;
}

i32 test_modules() {
	mut_i32 result = 0;
	result += test_stle_read_file();
	result += test_stle_trie();
	return result;
}
