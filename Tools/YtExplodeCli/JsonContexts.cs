using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using System.Collections.Generic;

[JsonSourceGenerationOptions(WriteIndented = true)]
[JsonSerializable(typeof(VideoInfo))]
[JsonSerializable(typeof(List<VideoSearchResultLite>))]
[JsonSerializable(typeof(DownloadInfo))]
internal partial class JsonContext : JsonSerializerContext
{
}
