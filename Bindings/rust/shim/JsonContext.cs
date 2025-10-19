using System.Text.Json;
using System.Text.Json.Serialization;

/// <summary>
/// JSON serialization context for NativeAOT compatibility.
/// This provides source-generated serializers for types used in the FFI layer.
/// </summary>
[JsonSourceGenerationOptions(
    WriteIndented = false,
    PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    DefaultIgnoreCondition = JsonIgnoreCondition.Never,
    IncludeFields = false)]
[JsonSerializable(typeof(JsonElement))]
[JsonSerializable(typeof(JsonDocument))]
[JsonSerializable(typeof(object))]
internal partial class JsonContext : JsonSerializerContext
{
}
