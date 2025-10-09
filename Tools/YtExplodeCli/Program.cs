using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Text.Json;
using YoutubeExplode;
using YoutubeExplode.Videos.Streams;
using YoutubeExplode.Search;

if (args.Length < 1)
{
    Console.WriteLine("Usage: <command> [options]");
    Console.WriteLine("Commands: info <url>, search <query> [--limit N], download <url> [--audio-only] [--output PATH]");
    return;
}

var command = args[0].ToLower();
var yt = new YoutubeClient();

switch (command)
{
    case "info":
        if (args.Length < 2) { Console.WriteLine("Error: Missing video URL"); return; }
        await InfoAsync(args[1]);
        break;

    case "search":
        if (args.Length < 2) { Console.WriteLine("Error: Missing search query"); return; }
        int limit = 10;
        if (args.Length >= 4 && args[2] == "--limit" && int.TryParse(args[3], out var l)) limit = l;
        await SearchAsync(args[1], limit);
        break;

    case "download":
        if (args.Length < 2) { Console.WriteLine("Error: Missing video URL"); return; }
        bool audioOnly = args.Contains("--audio-only");
        string? output = null;
        var outputIndex = Array.IndexOf(args, "--output");
        if (outputIndex != -1 && args.Length > outputIndex + 1) output = args[outputIndex + 1];
        await DownloadAsync(args[1], audioOnly, output);
        break;

    default:
        Console.WriteLine("Unknown command: " + command);
        break;
}

// ---------------------------
// Commands
// ---------------------------

async Task InfoAsync(string url)
{
    var video = await yt.Videos.GetAsync(url);

    var info = new VideoInfo
    {
        Id = video.Id.Value,
        Title = video.Title,
        Author = video.Author.ChannelTitle,
        Duration = video.Duration?.ToString(),
        UploadDate = video.UploadDate.ToString("yyyy-MM-dd"),
        Views = video.Engagement.ViewCount,
        Likes = video.Engagement.LikeCount,
        Description = video.Description
    };

    Console.WriteLine(JsonSerializer.Serialize(info, JsonContext.Default.VideoInfo));
}

async Task SearchAsync(string query, int limit)
{
    var resultsList = new List<VideoSearchResult>();
    await foreach (var video in yt.Search.GetVideosAsync(query))
    {
        resultsList.Add(video);
        if (resultsList.Count >= limit) break;
    }

    var liteResults = new List<VideoSearchResultLite>();
    foreach (var v in resultsList)
    {
        liteResults.Add(new VideoSearchResultLite
        {
            Id = v.Id.Value,
            Title = v.Title,
            Author = v.Author.ChannelTitle,
            Duration = v.Duration?.ToString()
        });
    }

    Console.WriteLine(JsonSerializer.Serialize(liteResults, JsonContext.Default.ListVideoSearchResultLite));
}

async Task DownloadAsync(string url, bool audioOnly, string? output)
{
    var video = await yt.Videos.GetAsync(url);
    var manifest = await yt.Videos.Streams.GetManifestAsync(video.Id);

    IStreamInfo stream;
    string ext;

    if (audioOnly)
    {
        stream = manifest.GetAudioOnlyStreams().GetWithHighestBitrate();
        ext = ".mp3";
    }
    else
    {
        stream = manifest.GetMuxedStreams().GetWithHighestVideoQuality();
        ext = ".mp4";
    }

    var fileName = $"{SanitizeFileName(video.Title)}{ext}";
    var outputPath = string.IsNullOrEmpty(output)
        ? Path.Combine(Environment.CurrentDirectory, fileName)
        : Directory.Exists(output)
            ? Path.Combine(output, fileName)
            : output;

    Console.WriteLine($"Downloading to: {outputPath}");
    await yt.Videos.Streams.DownloadAsync(stream, outputPath);
    Console.WriteLine("Download complete!");

    var info = new DownloadInfo { FileName = fileName, OutputPath = outputPath };
    Console.WriteLine(JsonSerializer.Serialize(info, JsonContext.Default.DownloadInfo));
}

string SanitizeFileName(string name)
{
    foreach (var c in Path.GetInvalidFileNameChars())
        name = name.Replace(c, '_');
    return name;
}
