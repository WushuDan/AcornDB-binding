using System.Text.Json.Serialization;

namespace AcornDB.Shim;

[JsonSerializable(typeof(object))]
[JsonSerializable(typeof(string))]
[JsonSerializable(typeof(int))]
[JsonSerializable(typeof(long))]
[JsonSerializable(typeof(double))]
[JsonSerializable(typeof(bool))]
[JsonSerializable(typeof(System.Text.Json.JsonElement))]
[JsonSerializable(typeof(System.Collections.Generic.Dictionary<string, object>))]
[JsonSerializable(typeof(System.Collections.Generic.List<object>))]
[JsonSerializable(typeof(System.Collections.Generic.List<string>))]
[JsonSerializable(typeof(System.Collections.Generic.List<int>))]
[JsonSerializable(typeof(System.Collections.Generic.List<double>))]
[JsonSerializable(typeof(System.Collections.Generic.List<bool>))]
[JsonSerializable(typeof(System.Collections.Generic.Dictionary<string, string>))]
[JsonSerializable(typeof(System.Collections.Generic.Dictionary<string, int>))]
[JsonSerializable(typeof(System.Collections.Generic.Dictionary<string, double>))]
[JsonSerializable(typeof(System.Collections.Generic.Dictionary<string, bool>))]
public partial class JsonContext : JsonSerializerContext
{
}