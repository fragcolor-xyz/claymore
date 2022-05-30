#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <shards.h>

#define ExternalVar struct SHVar
#define Var struct SHVar

typedef struct Option_ScriptEnv Option_ScriptEnv;

typedef struct GetDataRequest {
  ClonedVar wire;
  ExternalVar hash;
  ExternalVar result;
  struct Option_ScriptEnv env;
} GetDataRequest;

enum PollState_Tag
#ifdef __cplusplus
  : uint8_t
#endif // __cplusplus
 {
  Running,
  Failed,
  Finished,
};
#ifndef __cplusplus
typedef uint8_t PollState_Tag;
#endif // __cplusplus

typedef union PollState {
  PollState_Tag tag;
  struct {
    PollState_Tag failed_tag;
    ClonedVar failed;
  };
  struct {
    PollState_Tag finished_tag;
    ClonedVar finished;
  };
} PollState;

typedef struct UploadRequest {
  ClonedVar wire;
  ExternalVar node;
  ExternalVar signer_key;
  ExternalVar auth_key;
  ExternalVar proto_type;
  ExternalVar data;
  struct Option_ScriptEnv env;
} UploadRequest;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct GetDataRequest *clmrGetDataStart(const uint8_t *fragment_hash);

void clmrGetDataFree(struct GetDataRequest *request);

bool clmrPoll(WireRef wire, union PollState **output);

void clmrPollFree(union PollState *state);

struct UploadRequest *clmrUpload(const Var *var);

void clmrUploadFree(struct UploadRequest *request);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
