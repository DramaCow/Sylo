#ifndef __LEXER_H__
#define __LEXER_H__

#include <stddef.h>
#include <stdint.h>

struct RegEx_Lexer {
    const uint8_t *const input;
    const size_t length;
    size_t index;
};

enum RegEx_Lexer_TokenType {
    STRING,
    CHAR,
    RANGE,
    AND,
    OR,
    DIFF,
    OPT,
    STAR,
    PLUS,
    NOT,
    LPAREN,
    RPAREN,
    TT_ERROR = -1,
    TT_SKIP = -2,
};

struct RegEx_Lexer_Token {
    enum RegEx_Lexer_TokenType type;
    size_t span_start;
    size_t span_end;
};

struct RegEx_Lexer_Error {
    size_t pos;
};

struct RegEx_Lexer_Item {
    enum { OK = 0, ERR = 1, NONE = -1 } tag;
    union {
        struct RegEx_Lexer_Token token;
        struct RegEx_Lexer_Error error;
    };
};

struct RegEx_Lexer RegEx_Lexer_new(const uint8_t *input, size_t length);

struct RegEx_Lexer_Item RegEx_Lexer_next(struct RegEx_Lexer *const this);

#endif