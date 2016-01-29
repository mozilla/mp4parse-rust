/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include <cassert>
#include <cinttypes>
#include <cstdint>
#include <cstdio>
#include <vector>

#include "mp4parse.h"

void test_context()
{
  mp4parse_state *context = mp4parse_new();
  assert(context != nullptr);
  mp4parse_free(context);
}

void test_arg_validation(mp4parse_state *context)
{
  int32_t rv;

  rv = mp4parse_read(nullptr, nullptr, 0);
  assert(rv == MP4PARSE_ERROR_BADARG);

  rv = mp4parse_read(context, nullptr, 0);
  assert(rv == MP4PARSE_ERROR_BADARG);

  size_t len = 4097;
  rv = mp4parse_read(context, nullptr, len);
  assert(rv == MP4PARSE_ERROR_BADARG);

  std::vector<uint8_t> buf;
  rv = mp4parse_read(context, buf.data(), buf.size());
  assert(rv == MP4PARSE_ERROR_BADARG);

  buf.reserve(len);
  rv = mp4parse_read(context, buf.data(), buf.size());
  assert(rv == MP4PARSE_ERROR_BADARG);
}

void test_arg_validation()
{
  test_arg_validation(nullptr);

  mp4parse_state *context = mp4parse_new();
  assert(context != nullptr);
  test_arg_validation(context);
  mp4parse_free(context);
}

const char * tracktype2str(uint32_t type)
{
  switch (type) {
    case MP4PARSE_TRACK_TYPE_VIDEO: return "video";
    case MP4PARSE_TRACK_TYPE_AUDIO: return "audio";
  }
  return "unknown";
}

const char * errorstring(int32_t error)
{
  if (error >= MP4PARSE_OK) {
    return "Ok";
  }
  switch (error) {
    case MP4PARSE_ERROR_BADARG: return "Invalid argument";
    case MP4PARSE_ERROR_INVALID: return "Invalid data";
    case MP4PARSE_ERROR_UNSUPPORTED: return "Feature unsupported";
    case MP4PARSE_ERROR_EOF: return "Unexpected end-of-file";
    case MP4PARSE_ERROR_IO: return "I/O error";
  }
  return "Unknown error";
}

int32_t read_file(const char* filename)
{
  FILE* f = fopen(filename, "rb");
  assert(f != nullptr);

  size_t len = 4096*16;
  std::vector<uint8_t> buf(len);
  size_t read = fread(buf.data(), sizeof(decltype(buf)::value_type), buf.size(), f);
  buf.resize(read);
  fclose(f);

  mp4parse_state *context = mp4parse_new();
  assert(context != nullptr);

  fprintf(stderr, "Parsing %lu byte buffer.\n", (unsigned long)read);
  int32_t rv = mp4parse_read(context, buf.data(), buf.size());
  if (rv != MP4PARSE_OK) {
    fprintf(stderr, "Parsing failed: %s\n", errorstring(rv));
    return rv;
  }
  uint32_t tracks = mp4parse_get_track_count(context);
  fprintf(stderr, "%u tracks returned to C code.\n", tracks);

  for (uint32_t i = 0; i < tracks; ++i) {
    mp4parse_track_info track_info;
    int32_t rv2 = mp4parse_get_track_info(context, i, &track_info);
    assert(rv2 == MP4PARSE_OK);
    fprintf(stderr, "Track %d: type=%s duration=%" PRId64 " media_time=%" PRId64 " track_id=%d\n",
            i, tracktype2str(track_info.track_type), track_info.duration, track_info.media_time, track_info.track_id);
  }

  mp4parse_free(context);

  return MP4PARSE_OK;
}

int main(int argc, char* argv[])
{
  test_context();
  test_arg_validation();

  for (auto i = 1; i < argc; ++i) {
    read_file(argv[i]);
  }

  return 0;
}
