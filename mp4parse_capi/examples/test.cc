/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#undef NDEBUG
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
  mp4parse_io io = { abort_read, &dummy_value };
  mp4parse_parser *parser = mp4parse_new(&io);
  assert(parser != nullptr);
  mp4parse_free(parser);
  assert(dummy_value == 42);
}

void test_arg_validation()
{
  mp4parse_parser *parser = mp4parse_new(nullptr);
  assert(parser == nullptr);

  mp4parse_io io = { nullptr, nullptr };
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
  assert(rv == mp4parse_status_BAD_ARG);

  mp4parse_track_info info;
  rv = mp4parse_get_track_info(nullptr, 0, &info);
  assert(rv == mp4parse_status_BAD_ARG);

  mp4parse_track_video_info video;
  rv = mp4parse_get_track_video_info(nullptr, 0, &video);
  assert(rv == mp4parse_status_BAD_ARG);

  mp4parse_track_audio_info audio;
  rv = mp4parse_get_track_audio_info(nullptr, 0, &audio);
  assert(rv == mp4parse_status_BAD_ARG);

  assert(dummy_value == 42);
}

void test_arg_validation_with_parser()
{
  int dummy_value = 42;
  mp4parse_io io = { error_read, &dummy_value };
  mp4parse_parser *parser = mp4parse_new(&io);
  assert(parser != nullptr);

  int32_t rv = mp4parse_read(parser);
  assert(rv == mp4parse_status_IO);

  rv = mp4parse_get_track_info(parser, 0, nullptr);
  assert(rv == mp4parse_status_BAD_ARG);

  rv = mp4parse_get_track_video_info(parser, 0, nullptr);
  assert(rv == mp4parse_status_BAD_ARG);

  rv = mp4parse_get_track_audio_info(parser, 0, nullptr);
  assert(rv == mp4parse_status_BAD_ARG);

  mp4parse_free(parser);
  assert(dummy_value == 42);
}

void test_arg_validation_with_data(const std::string& filename)
{
  FILE* f = fopen(filename.c_str(), "rb");
  assert(f != nullptr);
  mp4parse_io io = { io_read, f };
  mp4parse_parser *parser = mp4parse_new(&io);
  assert(parser != nullptr);

  mp4parse_status rv = mp4parse_read(parser);
  assert(rv == mp4parse_status_OK);

  uint32_t tracks;
  rv = mp4parse_get_track_count(parser, &tracks);
  assert(rv == mp4parse_status_OK);
  assert(tracks == 2);

  mp4parse_track_info info;
  rv = mp4parse_get_track_info(parser, 0, &info);
  assert(rv == mp4parse_status_OK);
  assert(info.track_type == mp4parse_track_type_VIDEO);
  assert(info.track_id == 1);
  assert(info.duration == 40000);
  assert(info.media_time == 0);

  rv = mp4parse_get_track_info(parser, 1, &info);
  assert(rv == mp4parse_status_OK);
  assert(info.track_type == mp4parse_track_type_AUDIO);
  assert(info.track_id == 2);
  assert(info.duration == 61333);
  assert(info.media_time == 21333);

  mp4parse_track_video_info video;
  rv = mp4parse_get_track_video_info(parser, 0, &video);
  assert(rv == mp4parse_status_OK);
  assert(video.display_width == 320);
  assert(video.display_height == 240);
  assert(video.image_width == 320);
  assert(video.image_height == 240);

  mp4parse_track_audio_info audio;
  rv = mp4parse_get_track_audio_info(parser, 1, &audio);
  assert(rv == mp4parse_status_OK);
  assert(audio.channels == 1);
  assert(audio.bit_depth == 16);
  assert(audio.sample_rate == 48000);

  // Test with an invalid track number.

  rv = mp4parse_get_track_info(parser, 3, &info);
  assert(rv == mp4parse_status_BAD_ARG);
  rv = mp4parse_get_track_video_info(parser, 3, &video);
  assert(rv == mp4parse_status_BAD_ARG);
  rv = mp4parse_get_track_audio_info(parser, 3, &audio);
  assert(rv == mp4parse_status_BAD_ARG);

  mp4parse_free(parser);
  fclose(f);
}

const char * tracktype2str(mp4parse_track_type type)
{
  switch (type) {
    case mp4parse_track_type_VIDEO: return "video";
    case mp4parse_track_type_AUDIO: return "audio";
  }
  return "unknown";
}

const char * errorstring(mp4parse_status error)
{
  switch (error) {
    case mp4parse_status_OK: return "Ok";
    case mp4parse_status_BAD_ARG: return "Invalid argument";
    case mp4parse_status_INVALID: return "Invalid data";
    case mp4parse_status_UNSUPPORTED: return "Feature unsupported";
    case mp4parse_status_EOF: return "Unexpected end-of-file";
    case mp4parse_status_IO: return "I/O error";
    case mp4parse_status_TABLE_TOO_LARGE: return "Table too large";
  }
  return "Unknown error";
}

int32_t read_file(const char* filename)
{
  FILE* f = fopen(filename, "rb");
  assert(f != nullptr);

  mp4parse_io io = { io_read, f };
  mp4parse_parser *parser = mp4parse_new(&io);
  assert(parser != nullptr);

  fprintf(stderr, "Parsing file '%s'.\n", filename);
  mp4parse_status rv = mp4parse_read(parser);
  if (rv != mp4parse_status_OK) {
    mp4parse_free(parser);
    fclose(f);
    fprintf(stderr, "Parsing failed: %s\n", errorstring(rv));
    return rv;
  }
  uint32_t tracks;
  rv = mp4parse_get_track_count(parser, &tracks);
  assert(rv == mp4parse_status_OK);
  fprintf(stderr, "%u tracks returned to C code.\n", tracks);

  for (uint32_t i = 0; i < tracks; ++i) {
    mp4parse_track_info track_info;
    int32_t rv2 = mp4parse_get_track_info(parser, i, &track_info);
    assert(rv2 == mp4parse_status_OK);
    fprintf(stderr, "Track %d: type=%s duration=%" PRId64 " media_time=%" PRId64 " track_id=%d\n",
            i, tracktype2str(track_info.track_type), track_info.duration, track_info.media_time, track_info.track_id);
  }

  mp4parse_free(parser);
  fclose(f);

  return mp4parse_status_OK;
}

int main(int argc, char* argv[])
{
  test_new_parser();
  test_arg_validation();
  test_arg_validation_with_parser();

  // Find our test file relative to our executable file path.
  std::string path(realpath(argv[0], NULL));
  auto split = path.rfind('/');
  path.replace(split, path.length() - split, "/../../mp4parse/tests/minimal.mp4");
  test_arg_validation_with_data(path);

  // Run any other test files passed on the command line.
  for (auto i = 1; i < argc; ++i) {
    read_file(argv[i]);
  }

  return 0;
}
