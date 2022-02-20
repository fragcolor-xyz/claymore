#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <chainblocks.h>

#define ExternalVar struct CBVar
#define Var struct CBVar

typedef struct Option_ScriptEnv Option_ScriptEnv;

typedef struct GetDataRequest {
  ClonedVar chain;
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

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct GetDataRequest *clmrGetDataStart(const uint8_t *fragment_hash);

void clmrGetDataFree(struct GetDataRequest *request);

bool clmrPoll(const Var *chain, union PollState **output);

void clmrPollFree(union PollState *state);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
