@0x84928e7acd17ba2f;

struct GraphHeader {
       tag @0 : Text;
       numNodes @1 : UInt32;
       numEdges @2 : UInt64;
}

struct Edge {
       from @0 : UInt32;
       to @1 : UInt32;
       weight @2 : Float32;
}