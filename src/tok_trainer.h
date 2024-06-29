#pragma once
#include "stle.h"


// typedef struct Range {
// 	u32 *bounds[2];
// } Range;

ErrorCode tok_trainer_train_unigram_bytes(TrieNode *root, Bytes *bytes);
ErrorCode test_tok_trainer_train_unigram_bytes();
ErrorCode test_tok_trainer_modules();
