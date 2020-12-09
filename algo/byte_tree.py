import matplotlib.pyplot as plt
import numpy as np
from graphviz import Digraph


class Node:
    def __init__(self, accept=False):
        self.accept = accept
        self.edges = []
        self.children = []

    @property
    def is_leaf(self):
        return len(self.children) == 0

    def add_edge(self, edge, child):
        self.edges.append(edge)
        self.children.append(child)

    def match(self, word):
        num_bytes = 1
        while word >> (8 * num_bytes) != 0:
            num_bytes += 1
        bytes = word.to_bytes(num_bytes, byteorder='little')
        print([hex(byte)[2:] for byte in bytes])
        return self._match_bytes(bytes)

    def _match_bytes(self, bytes):
        if len(bytes) == 0:
            return self.accept
        head, tail = bytes[0], bytes[1:]
        # print('head:', head, ', tail:', tail)
        for edge, child in zip(self.edges, self.children):
            if head in range(edge[0], edge[1] + 1):
                return child._match_bytes(tail)
        return False        

    def to_graph(self):
        graph = Digraph()
        self._to_graph_internal(graph, 0)
        return graph

    def _to_graph_internal(self, graph, id):
        graph.node('node%d' % id, label='Acc' if self.accept else '')
        next_id = id + 1
        for j, child in enumerate(self.children):
            child_id = next_id
            next_id = child._to_graph_internal(graph, next_id)
            graph.edge('node%d' % id, 'node%d' % child_id, label=to_hex(self.edges[j]))
        return next_id


def byte_tree(a, b, is_root=False):
    # 0 alone constitutes one byte. As such, the root cannot accept (else
    # the empty bytestring is accepted). This is handled by forcing the
    # root node to be non-accepting.
    node = Node(a == 0 and not is_root)
    # node = Node(b == 0) # use this condition if leading zeros is valid.

    a_low, a_high = (a & 0xFF, a >> 8)
    b_low, b_high = (b & 0xFF, b >> 8)

    # Forces the leading byte to be nonzero.
    if node.accept:
        a_low += 1

    # +--------------+--------------+--------------+--------------+
    # | 1: . . . . . | 2: . . . . . | 3: . . . . . | 4: . . . . . |
    # |    . . . . . |    . a-->b . |    ------->b |    --->b . . |
    # |    . a-->b . |    . . . . . |    a-------> |    a-------> |
    # +--------------+--------------+--------------+--------------+
    # | 5: . . . . . | 6: ----->b . | 7: ->b . . . | 8: . . . . . |
    # |    ------->b |    --------> |    --------> |    ->b . . . |
    # |    . . a---> |    . a-----> |    . . . a-> |    . . a---> |
    # +--------------+--------------+--------------+--------------+
    # [ Row represents high value, column represents low byte. ]

    # 1
    if b_high == 0:
        node.add_edge((a_low, b_low), Node(True))
    # 2
    elif a_high == b_high:
        node.add_edge((a_low, b_low), byte_tree(a_high, b_high))
    # 3
    elif a_low == 0x00 and b_low == 0xFF:
        node.add_edge((0x00, 0xFF), byte_tree(a_high, b_high))
    # 4
    elif a_low == 0x00:
        node.add_edge((0x00, b_low), byte_tree(a_high, b_high))
        node.add_edge((b_low + 1, 0xFF), byte_tree(a_high, b_high - 1))
    # 5
    elif b_low == 0xFF:
        node.add_edge((0x00, a_low - 1), byte_tree(a_high + 1, b_high))
        node.add_edge((a_low, 0xFF), byte_tree(a_high, b_high))
    # 6
    elif b_low >= a_low:
        node.add_edge((0x00, a_low - 1), byte_tree(a_high + 1, b_high))
        node.add_edge((a_low, b_low), byte_tree(a_high, b_high))
        node.add_edge((b_low + 1, 0xFF), byte_tree(a_high, b_high - 1))
    # 7
    elif b_high > a_high + 1 and b_low < a_low - 1:
        node.add_edge((0x00, b_low), byte_tree(a_high + 1, b_high))
        node.add_edge((b_low + 1, a_low - 1), byte_tree(a_high + 1, b_high - 1))
        node.add_edge((a_low, 0xFF), byte_tree(a_high, b_high - 1))
    # 8
    else:
        node.add_edge((0x00, b_low), byte_tree(a_high + 1, b_high))
        node.add_edge((a_low, 0xFF), byte_tree(a_high, b_high - 1))

    return node


def to_hex(x):
    if isinstance(x, str):
        return x
    if isinstance(x, tuple):
        if x[0] != x[1]:
            return hex(x[0])[2:] + '..' + hex(x[1])[2:]
        return hex(x[0])[2:]
    return hex(x)[2:]


if __name__=='__main__':
    # tree = byte_tree(0x1, 0xDEADBEEF, is_root=True)
    # assert not tree.match(0)
    # assert not tree.match(0x0)
    # assert tree.match(0x1)
    # assert tree.match(0xDEADBEEF)
    # assert not tree.match(0xDEADBEEF + 1)
    # assert not tree.match(0xFFFFFFFF)

    low = 0x0000
    high = 0xDEADBEEF
    print(high - low)
    tree = byte_tree(low, high, is_root=True)
    print(tree.to_graph().render('tmp.gv', view=True))