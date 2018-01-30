/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#undef NDEBUG
#include <algorithm>
#include <cassert>
#include <cinttypes>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include "mp4parse.h"

intptr_t abort_read(uint8_t *buffer, uintptr_t size, void *userdata)
{
  // This shouldn't be called when allocating a parser.
  abort();
}

intptr_t error_read(uint8_t *buffer, uintptr_t size, void *userdata)
{
  return -1;
}

intptr_t io_read(uint8_t *buffer, uintptr_t size, void *userdata)
{
  FILE *f = reinterpret_cast<FILE *>(userdata);

  size_t r = fread(buffer, 1, size, f);
  if (r == 0 && feof(f))
    return 0;
  if (r == 0 && ferror(f))
    return -1;
  return r;
}

void test_new_parser()
{
  int dummy_value = 42;
  Mp4parseIo io = { abort_read, &dummy_value };
  Mp4parseParser *parser = mp4parse_new(&io);
  assert(parser != nullptr);
  mp4parse_free(parser);
  assert(dummy_value == 42);
}

void test_arg_validation()
{
  Mp4parseParser *parser = mp4parse_new(nullptr);
  assert(parser == nullptr);

  Mp4parseIo io = { nullptr, nullptr };
  parser = mp4parse_new(&io);
  assert(parser == nullptr);

  io = { abort_read, nullptr };
  parser = mp4parse_new(&io);
  assert(parser == nullptr);

  int dummy_value = 42;
  io = { nullptr, &dummy_value };
  parser = mp4parse_new(&io);
  assert(parser == nullptr);

  int32_t rv = mp4parse_read(nullptr);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  Mp4parseTrackInfo info;
  rv = mp4parse_get_track_info(nullptr, 0, &info);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  Mp4parseTrackVideoInfo video;
  rv = mp4parse_get_track_video_info(nullptr, 0, &video);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  Mp4parseTrackAudioInfo audio;
  rv = mp4parse_get_track_audio_info(nullptr, 0, &audio);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  assert(dummy_value == 42);
}

void test_arg_validation_with_parser()
{
  int dummy_value = 42;
  Mp4parseIo io = { error_read, &dummy_value };
  Mp4parseParser *parser = mp4parse_new(&io);
  assert(parser != nullptr);

  int32_t rv = mp4parse_read(parser);
  assert(rv == MP4PARSE_STATUS_IO);

  rv = mp4parse_get_track_info(parser, 0, nullptr);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  rv = mp4parse_get_track_video_info(parser, 0, nullptr);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  rv = mp4parse_get_track_audio_info(parser, 0, nullptr);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  mp4parse_free(parser);
  assert(dummy_value == 42);
}

void test_arg_validation_with_data(const std::string& filename)
{
  FILE* f = fopen(filename.c_str(), "rb");
  assert(f != nullptr);
  Mp4parseIo io = { io_read, f };
  Mp4parseParser *parser = mp4parse_new(&io);
  assert(parser != nullptr);

  Mp4parseStatus rv = mp4parse_read(parser);
  assert(rv == MP4PARSE_STATUS_OK);

  uint32_t tracks;
  rv = mp4parse_get_track_count(parser, &tracks);
  assert(rv == MP4PARSE_STATUS_OK);
  assert(tracks == 2);

  Mp4parseTrackInfo info;
  rv = mp4parse_get_track_info(parser, 0, &info);
  assert(rv == MP4PARSE_STATUS_OK);
  assert(info.track_type == MP4PARSE_TRACK_TYPE_VIDEO);
  assert(info.track_id == 1);
  assert(info.duration == 40000);
  assert(info.media_time == 0);

  rv = mp4parse_get_track_info(parser, 1, &info);
  assert(rv == MP4PARSE_STATUS_OK);
  assert(info.track_type == MP4PARSE_TRACK_TYPE_AUDIO);
  assert(info.track_id == 2);
  assert(info.duration == 61333);
  assert(info.media_time == 21333);

  Mp4parseTrackVideoInfo video;
  rv = mp4parse_get_track_video_info(parser, 0, &video);
  assert(rv == MP4PARSE_STATUS_OK);
  assert(video.display_width == 320);
  assert(video.display_height == 240);
  assert(video.image_width == 320);
  assert(video.image_height == 240);

  Mp4parseTrackAudioInfo audio;
  rv = mp4parse_get_track_audio_info(parser, 1, &audio);
  assert(rv == MP4PARSE_STATUS_OK);
  assert(audio.channels == 1);
  assert(audio.bit_depth == 16);
  assert(audio.sample_rate == 48000);

  // Test with an invalid track number.

  rv = mp4parse_get_track_info(parser, 3, &info);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);
  rv = mp4parse_get_track_video_info(parser, 3, &video);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);
  rv = mp4parse_get_track_audio_info(parser, 3, &audio);
  assert(rv == MP4PARSE_STATUS_BAD_ARG);

  mp4parse_free(parser);
  fclose(f);
}

const char * tracktype2str(Mp4parseTrackType type)
{
  switch (type) {
    case MP4PARSE_TRACK_TYPE_VIDEO: return "video";
    case MP4PARSE_TRACK_TYPE_AUDIO: return "audio";
  }
  return "unknown";
}

const char * errorstring(Mp4parseStatus error)
{
  switch (error) {
    case MP4PARSE_STATUS_OK: return "Ok";
    case MP4PARSE_STATUS_BAD_ARG: return "Invalid argument";
    case MP4PARSE_STATUS_INVALID: return "Invalid data";
    case MP4PARSE_STATUS_UNSUPPORTED: return "Feature unsupported";
    case MP4PARSE_STATUS_EOF: return "Unexpected end-of-file";
    case MP4PARSE_STATUS_IO: return "I/O error";
    case MP4PARSE_STATUS_OOM: return "Out of memory";
  }
  return "Unknown error";
}

int32_t read_file(const char* filename)
{
  FILE* f = fopen(filename, "rb");
  assert(f != nullptr);

  Mp4parseIo io = { io_read, f };
  Mp4parseParser *parser = mp4parse_new(&io);
  assert(parser != nullptr);

  fprintf(stderr, "Parsing file '%s'.\n", filename);
  Mp4parseStatus rv = mp4parse_read(parser);
  if (rv != MP4PARSE_STATUS_OK) {
    mp4parse_free(parser);
    fclose(f);
    fprintf(stderr, "Parsing failed: %s\n", errorstring(rv));
    return rv;
  }
  uint32_t tracks;
  rv = mp4parse_get_track_count(parser, &tracks);
  assert(rv == MP4PARSE_STATUS_OK);
  fprintf(stderr, "%u tracks returned to C code.\n", tracks);

  for (uint32_t i = 0; i < tracks; ++i) {
    Mp4parseTrackInfo track_info;
    int32_t rv2 = mp4parse_get_track_info(parser, i, &track_info);
    assert(rv2 == MP4PARSE_STATUS_OK);
    fprintf(stderr, "Track %d: type=%s duration=%" PRId64 " media_time=%" PRId64 " track_id=%d\n",
            i, tracktype2str(track_info.track_type), track_info.duration, track_info.media_time, track_info.track_id);
  }

  mp4parse_free(parser);
  fclose(f);

  return MP4PARSE_STATUS_OK;
}

int main(int argc, char* argv[])
{
  // Parse command line options.
  std::vector<std::string> args(argv + 1, argv + argc);
  args.erase(
    std::remove_if(args.begin(), args.end(), [](std::string& arg){
      if (!arg.compare("-v")) {
        fprintf(stderr, "Enabling debug logging.\n");
        const char* LOG_ENV = "RUST_LOG";
        auto logger = std::string(getenv(LOG_ENV));
        if (!logger.empty()) {
          logger.append(",");
        }
        logger.append("debug");
        setenv(LOG_ENV, logger.c_str(), 1);
        return true;
      }
      return false;
    }),
    args.end()
  );

  test_new_parser();
  test_arg_validation();
  test_arg_validation_with_parser();

  // Find our test file relative to our executable file path.
  char* real = realpath(argv[0], NULL);
  std::string path(real);
  free(real);
  auto split = path.rfind('/');
  path.replace(split, path.length() - split, "/../../mp4parse/tests/minimal.mp4");
  test_arg_validation_with_data(path);

  // Run any other test files passed on the command line.
  for (auto arg: args) {
    read_file(arg.c_str());
  }

  return 0;
}
