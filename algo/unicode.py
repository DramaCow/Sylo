class Range:
    def __init__(self, low, high):
        self.low = low
        self.high = high

def range_diff(a, b):
    # disjoint
    if a.high < b.low or b.high < a.low:
        return [a]
    # b covers a
    if b.low <= a.low and a.high <= b.high:
        return []
    # a covers b
    if a.low <= b.low and b.high <= a.high:
        ranges = []
        if a.low != b.low: ranges.append(Range(a.low, b.low - 1))
        if b.high != a.high: ranges.append(Range(b.high + 1, a.high))
        return ranges
    # b.low in a
    if a.high >= b.low:
        return [(a.low, b.low - 1)] if a.low != b.low else []
    # b.high in a
    return [(b.high + 1, a.high)] if b.high != a.high else []
    
# ref: https://www.w3.org/TR/xml/#NT-Char
# Char	  ::= ( #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF] ) - Compat
# Compat ::= [#x7F-#x84]         | [#x86-#x9F]       | [#xFDD0-#xFDEF]   |
#            [#x1FFFE-#x1FFFF]   | [#x2FFFE-#x2FFFF] | [#x3FFFE-#x3FFFF] |
#            [#x4FFFE-#x4FFFF]   | [#x5FFFE-#x5FFFF] | [#x6FFFE-#x6FFFF] |
#            [#x7FFFE-#x7FFFF]   | [#x8FFFE-#x8FFFF] | [#x9FFFE-#x9FFFF] |
#            [#xAFFFE-#xAFFFF]   | [#xBFFFE-#xBFFFF] | [#xCFFFE-#xCFFFF] |
#            [#xDFFFE-#xDFFFF]   | [#xEFFFE-#xEFFFF] | [#xFFFFE-#xFFFFF] |
#            [#x10FFFE-#x10FFFF]

chars = [
    Range(0x000009, 0x000009),
    Range(0x00000A, 0x00000A),
    Range(0x00000D, 0x00000D),
    Range(0x000020, 0x00D7FF),
    Range(0x00E000, 0x00FFFD),
    Range(0x010000, 0x10FFFF),
]

compat = [
    Range(0x00007F, 0x000084),
    Range(0x000086, 0x00009F),
    Range(0x00FDD0, 0x00FDEF),
    Range(0x01FFFE, 0x01FFFF),
    Range(0x02FFFE, 0x02FFFF),
    Range(0x03FFFE, 0x03FFFF),
    Range(0x04FFFE, 0x04FFFF),
    Range(0x05FFFE, 0x05FFFF),
    Range(0x06FFFE, 0x06FFFF),
    Range(0x07FFFE, 0x07FFFF),
    Range(0x08FFFE, 0x08FFFF),
    Range(0x09FFFE, 0x09FFFF),
    Range(0x0AFFFE, 0x0AFFFF),
    Range(0x0BFFFE, 0x0BFFFF),
    Range(0x0CFFFE, 0x0CFFFF),
    Range(0x0DFFFE, 0x0DFFFF),
    Range(0x0EFFFE, 0x0EFFFF),
    Range(0x0FFFFE, 0x0FFFFF),
    Range(0x10FFFE, 0x10FFFF),
]

for b in compat:
    chars = [char for a in chars for char in range_diff(a, b)]

for r in chars:
    low = '0x' + hex(r.low)[2:].upper().zfill(6)
    high = '0x' + hex(r.high)[2:].upper().zfill(6)
    print('.chain((' + low + '..=' + high + ').filter_map(from_u32))')        