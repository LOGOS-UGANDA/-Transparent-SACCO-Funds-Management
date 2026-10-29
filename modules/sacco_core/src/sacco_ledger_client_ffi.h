#pragma once

#ifdef __cplusplus
extern "C" {
#endif

char* sacco_ledger_encode_instruction_json(const char* instruction_json);
char* sacco_ledger_instruction_accounts_json(const char* instruction_name);
char* sacco_ledger_decode_shard_json(const char* request_json);
char* sacco_ledger_config_pda_json(const char* program_id_base58);
void sacco_ledger_client_free(char* value);

#ifdef __cplusplus
}
#endif
