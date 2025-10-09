public class VideoInfo
{
    public string Id { get; set; } = "";
    public string Title { get; set; } = "";
    public string Author { get; set; } = "";
    public string? Duration { get; set; }
    public string? UploadDate { get; set; }
    public long? Views { get; set; }
    public long? Likes { get; set; }
    public string Description { get; set; } = "";
}

public class VideoSearchResultLite
{
    public string Id { get; set; } = "";
    public string Title { get; set; } = "";
    public string Author { get; set; } = "";
    public string? Duration { get; set; }
}

public class DownloadInfo
{
    public string FileName { get; set; } = "";
    public string OutputPath { get; set; } = "";
}
