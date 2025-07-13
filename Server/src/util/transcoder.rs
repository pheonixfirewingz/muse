use ffmpeg_next as ffmpeg;
use ffmpeg::{codec, filter, format, media};
use anyhow::{Result, Context};
use std::path::Path;

/// Struct for managing the transcoding process.
struct Transcoder {
    stream: usize,
    filter: filter::Graph,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
    in_time_base: ffmpeg::Rational,
    out_time_base: ffmpeg::Rational,
}

/// Create a transcoder that copies sample rate and metadata from input.
fn transcoder<P: AsRef<Path> + ?Sized>(
    ictx: &mut format::context::Input,
    octx: &mut format::context::Output,
    path: &P,
    filter_spec: &str,
) -> Result<Transcoder, ffmpeg::Error> {
    let input = ictx
        .streams()
        .best(media::Type::Audio)
        .expect("could not find best audio stream");
    let context = codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context.decoder().audio()?;
    let codec = ffmpeg::encoder::find(octx.format().codec(path, media::Type::Audio))
        .expect("failed to find encoder")
        .audio()?;
    let global = octx
        .format()
        .flags()
        .contains(format::flag::Flags::GLOBAL_HEADER);

    decoder.set_parameters(input.parameters())?;

    let mut output = octx.add_stream(codec)?;
    let context = codec::context::Context::from_parameters(output.parameters())?;
    let mut encoder = context.encoder().audio()?;

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(decoder.channel_layout().channels()))
        .unwrap_or(ffmpeg::channel_layout::ChannelLayout::STEREO);

    if global {
        encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
    }

    // Copy sample rate from input
    encoder.set_rate(decoder.rate() as i32);
    encoder.set_channel_layout(channel_layout);
    encoder.set_format(
        codec
            .formats()
            .expect("unknown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(decoder.bit_rate());
    encoder.set_max_bit_rate(decoder.max_bit_rate());

    encoder.set_time_base((1, decoder.rate() as i32));
    output.set_time_base((1, decoder.rate() as i32));

    let encoder = encoder.open_as(codec)?;
    output.set_parameters(&encoder);

    // Filter graph setup
    let filter = {
        let mut filter = filter::Graph::new();

        let args = format!(
            "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
            decoder.time_base(),
            decoder.rate(),
            decoder.format().name(),
            decoder.channel_layout().bits()
        );

        filter.add(&filter::find("abuffer").unwrap(), "in", &args)?;
        filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;

        {
            let mut out = filter.get("out").unwrap();
            out.set_sample_format(encoder.format());
            out.set_channel_layout(encoder.channel_layout());
            out.set_sample_rate(encoder.rate());
        }

        filter.output("in", 0)?.input("out", 0)?.parse(filter_spec)?;
        filter.validate()?;

        if let Some(codec) = encoder.codec() {
            if !codec
                .capabilities()
                .contains(codec::capabilities::Capabilities::VARIABLE_FRAME_SIZE)
            {
                filter
                    .get("out")
                    .unwrap()
                    .sink()
                    .set_frame_size(encoder.frame_size());
            }
        }
        filter
    };

    let in_time_base = decoder.time_base();
    let out_time_base = output.time_base();

    Ok(Transcoder {
        stream: input.index(),
        filter,
        decoder,
        encoder,
        in_time_base,
        out_time_base,
    })
}

impl Transcoder {
    fn send_frame_to_encoder(&mut self, frame: &ffmpeg::Frame) {
        self.encoder.send_frame(frame).unwrap();
    }

    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(&mut self, octx: &mut format::context::Output) {
        let mut encoded = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            encoded.rescale_ts(self.in_time_base, self.out_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }

    fn add_frame_to_filter(&mut self, frame: &ffmpeg::Frame) {
        self.filter.get("in").unwrap().source().add(frame).unwrap();
    }

    fn flush_filter(&mut self) {
        self.filter.get("in").unwrap().source().flush().unwrap();
    }

    fn get_and_process_filtered_frames(&mut self, octx: &mut format::context::Output) {
        let mut filtered = ffmpeg::frame::Audio::empty();
        while self
            .filter
            .get("out")
            .unwrap()
            .sink()
            .frame(&mut filtered)
            .is_ok()
        {
            self.send_frame_to_encoder(&filtered);
            self.receive_and_process_encoded_packets(octx);
        }
    }

    fn send_packet_to_decoder(&mut self, packet: &ffmpeg::Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_and_process_decoded_frames(&mut self, octx: &mut format::context::Output) {
        let mut decoded = ffmpeg::frame::Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();
            decoded.set_pts(timestamp);
            self.add_frame_to_filter(&decoded);
            self.get_and_process_filtered_frames(octx);
        }
    }
}

/// Transcode to AAC, copying sample rate and all metadata.
pub fn transcode_to_aac_sync(input_path: &str, output_path: &str) -> Result<()> {
    ffmpeg::log::set_level(ffmpeg::log::Level::Error);
    ffmpeg::init().context("Failed to initialize ffmpeg")?;

    let mut ictx = format::input(&input_path)
        .context("Failed to open input file")?;

    let mut octx = format::output(&output_path)
        .context("Failed to open output file")?;

    // Copy global metadata
    octx.set_metadata(ictx.metadata().to_owned());

    // Use a no-op filter to preserve sample rate and format
    let filter_spec = "anull";

    let mut transcoder = transcoder(&mut ictx, &mut octx, Path::new(output_path), filter_spec)
        .context("Failed to create transcoder")?;

    octx.write_header().context("Failed to write output header")?;

    for (stream, packet) in ictx.packets() {
        if stream.index() == transcoder.stream {
            transcoder.send_packet_to_decoder(&packet);
            transcoder.receive_and_process_decoded_frames(&mut octx);
        }
    }

    transcoder.send_eof_to_decoder();
    transcoder.receive_and_process_decoded_frames(&mut octx);

    transcoder.flush_filter();
    transcoder.get_and_process_filtered_frames(&mut octx);

    transcoder.send_eof_to_encoder();
    transcoder.receive_and_process_encoded_packets(&mut octx);

    octx.write_trailer().context("Failed to write trailer")?;
    Ok(())
}

/// Async wrapper for use in async code with tokio
pub async fn transcode_to_aac(input_path: &str, output_path: &str) -> Result<()> {
    let input = input_path.to_owned();
    let output = output_path.to_owned();
    tokio::task::spawn_blocking(move || transcode_to_aac_sync(&input, &output)).await?
}