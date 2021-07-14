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

static inline struct RegEx_Lexer_Item RegEx_Lexer_Item_newToken(enum RegEx_Lexer_TokenType type, size_t span_start, size_t span_end) {
    return (struct RegEx_Lexer_Item) { .tag = OK, .token = { type, span_start, span_end } };
}

static inline struct RegEx_Lexer_Item RegEx_Lexer_Item_newError(size_t pos) {
    return (struct RegEx_Lexer_Item) { .tag = ERR, .error = pos };
}

static inline struct RegEx_Lexer_Item RegEx_Lexer_Item_newNone() {
    return (struct RegEx_Lexer_Item) { .tag = NONE };
}

struct RegEx_Lexer RegEx_Lexer_new(const uint8_t *input, size_t length) {
    return (struct RegEx_Lexer) { .input = input, .length = length, .index = 0 };
}

struct RegEx_Lexer_Item RegEx_Lexer_next(struct RegEx_Lexer *const this) {
    if (this->index >= this->length) {
        return RegEx_Lexer_Item_newNone();
    }

    size_t start_index = this->index;
    enum RegEx_Lexer_TokenType last_accept_ttype = TT_ERROR;
    size_t last_accept_index;

    // *** LEXER TABLE START ***
    s0: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        last_accept_ttype = TT_SKIP;
        last_accept_index = this->index;
        if (ch == 0x29) { ++this->index; goto s1; }
        if (ch == 0x21) { ++this->index; goto s2; }
        if (ch == 0x26) { ++this->index; goto s3; }
        if (ch == 0x09 ||
            ch == 0x0a ||
            ch == 0x0d ||
            ch == 0x20) { ++this->index; goto s4; }
        if (ch == 0x28) { ++this->index; goto s5; }
        if (ch == 0x2e) { ++this->index; goto s6; }
        if (ch == 0x22) { ++this->index; goto s8; }
        if (ch == 0x2a) { ++this->index; goto s12; }
        if (ch == 0x3f) { ++this->index; goto s13; }
        if (ch == 0x2d) { ++this->index; goto s14; }
        if (ch == 0x7c) { ++this->index; goto s15; }
        if (ch == 0x2b) { ++this->index; goto s16; }
        if (ch == 0x27) { ++this->index; goto s17; }
        return RegEx_Lexer_next(this);
    }
    s1: {
        return RegEx_Lexer_Item_newToken(RPAREN, start_index, this->index);
    }
    s2: {
        return RegEx_Lexer_Item_newToken(NOT, start_index, this->index);
    }
    s3: {
        return RegEx_Lexer_Item_newToken(AND, start_index, this->index);
    }
    s4: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x09 ||
            ch == 0x0a ||
            ch == 0x0d ||
            ch == 0x20) { ++this->index; goto s4; }
        return RegEx_Lexer_next(this);
    }
    s5: {
        return RegEx_Lexer_Item_newToken(LPAREN, start_index, this->index);
    }
    s6: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x2e) { ++this->index; goto s7; }
        goto sink;
    }
    s7: {
        return RegEx_Lexer_Item_newToken(RANGE, start_index, this->index);
    }
    s8: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x5c) { ++this->index; goto s9; }
        if (ch == 0x09 ||
            ch == 0x0a ||
            ch == 0x0d ||
            ch == 0x20 ||
            ch == 0x21 ||
            (0x23 <= ch && ch <= 0x5b) ||
            (0x5d <= ch && ch <= 0x7e) ||
            (0xa0 <= ch && ch <= 0xff)) { ++this->index; goto s10; }
        goto sink;
    }
    s9: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x22 ||
            ch == 0x5c) { ++this->index; goto s10; }
        goto sink;
    }
    s10: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x5c) { ++this->index; goto s9; }
        if (ch == 0x09 ||
            ch == 0x0a ||
            ch == 0x0d ||
            ch == 0x20 ||
            ch == 0x21 ||
            (0x23 <= ch && ch <= 0x5b) ||
            (0x5d <= ch && ch <= 0x7e) ||
            (0xa0 <= ch && ch <= 0xff)) { ++this->index; goto s10; }
        if (ch == 0x22) { ++this->index; goto s11; }
        goto sink;
    }
    s11: {
        return RegEx_Lexer_Item_newToken(STRING, start_index, this->index);
    }
    s12: {
        return RegEx_Lexer_Item_newToken(STAR, start_index, this->index);
    }
    s13: {
        return RegEx_Lexer_Item_newToken(OPT, start_index, this->index);
    }
    s14: {
        return RegEx_Lexer_Item_newToken(DIFF, start_index, this->index);
    }
    s15: {
        return RegEx_Lexer_Item_newToken(OR, start_index, this->index);
    }
    s16: {
        return RegEx_Lexer_Item_newToken(PLUS, start_index, this->index);
    }
    s17: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x09 ||
            ch == 0x0a ||
            ch == 0x0d ||
            ch == 0x20 ||
            ch == 0x21 ||
            (0x23 <= ch && ch <= 0x5b) ||
            (0x5d <= ch && ch <= 0x7e) ||
            (0xa0 <= ch && ch <= 0xff)) { ++this->index; goto s18; }
        if (ch == 0x5c) { ++this->index; goto s20; }
        goto sink;
    }
    s18: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x27) { ++this->index; goto s19; }
        goto sink;
    }
    s19: {
        return RegEx_Lexer_Item_newToken(CHAR, start_index, this->index);
    }
    s20: {
        if (this->index >= this->length) goto sink;
        uint8_t ch = this->input[this->index];
        if (ch == 0x22 ||
            ch == 0x5c) { ++this->index; goto s18; }
        goto sink;
    }
    // ***  LEXER TABLE END  ***

    sink:
        if (last_accept_ttype == TT_ERROR) {
            size_t pos = this->index;
            this->index = -1; // forces next iteration to return None
            return RegEx_Lexer_Item_newError(pos);
        } else if (last_accept_ttype == TT_SKIP) {
            this->index = last_accept_index;
            return RegEx_Lexer_next(this);
        } else {
            this->index = last_accept_index;
            return RegEx_Lexer_Item_newToken(last_accept_ttype, start_index, last_accept_index);
        }
}
