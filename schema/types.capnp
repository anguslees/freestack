@0xddaa8bf0e7dc1d47;

# Miscellaneous common types.

using Iso8601Date = Text;
# The date and time stamp format is ISO 8601:
#
# CCYY-MM-DDThh:mm:ss±hh:mm
# The ±hh:mm value, if included, returns the time zone as an offset from UTC.
#
# For example, 2015-08-27T09:49:58-05:00.
#
# If you omit the time zone, the UTC time zone is assumed.

using Url = Text;

struct KeyValue(K, V) {
  key @0 :K;
  value @1 :V;
}
# Can't do this in this file?
#  using Map(K, V) = List(KeyValue(K, V));
# produces:
#  error: 'using' declaration without '=' must specify a named declaration from a different scope.

using Metadata = List(KeyValue(Text, Text));

struct Result(R, E) {
  union {
    ok @0 :R;
    err @1 :E;
  }
}

interface Resource(Details) {
  details @0 () -> (details :Details);

  addDetailsListener @1 (l :DetailsListener);
  # The DetailsListener is called immediately (asynchronously) with
  # the initial details, and with every update thereafter.
  interface DetailsListener {
    update @0 (details :Details);
  }
}

interface DataProvider {
  read @0 (offset :UInt64, length :UInt64) -> (data :Data);
}

interface DataConsumer {
  append @0 (data :Data);
  flush @1 ();
  close @2 ();
}
