#!/usr/bin/env bash
set -euo pipefail
ffmpeg_version="8.1.3"
ffmpeg_sha256="7138d28c96d9d3e3af4ee3d8cad72741f8ffb40da90c1112235dea3ecd3178a3"
target="aarch64-apple-darwin"
minimum_macos="12.0"
project_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cache_directory="$project_directory/build/.cache/ffmpeg"
source_archive="$cache_directory/ffmpeg-$ffmpeg_version.tar.xz"
source_directory="$cache_directory/ffmpeg-$ffmpeg_version"
object_directory="$cache_directory/objects-$ffmpeg_version-$target"
binaries_directory="$project_directory/backend/binaries"
license_file="$project_directory/backend/licenses/FFmpeg-LGPL-2.1.txt"
architecture_flags="-arch arm64 -mmacosx-version-min=$minimum_macos"
configure_flags=(
  --arch=aarch64
  --target-os=darwin
  --cc=clang
  "--extra-cflags=$architecture_flags"
  "--extra-ldflags=$architecture_flags"
  --enable-static
  --disable-shared
  --disable-autodetect
  --disable-everything
  --disable-doc
  --disable-debug
  --disable-network
  --disable-avdevice
  --disable-ffplay
  --enable-ffmpeg
  --enable-ffprobe
  --enable-pthreads
  --enable-zlib
  --enable-videotoolbox
  --enable-protocol=file,pipe,subfile
  --enable-demuxer=mov,matroska,wav,h264
  --enable-decoder=h264,hevc,vp8,vp9,prores,mpeg4,pcm_s16le
  --enable-parser=h264,hevc,vp8,vp9,mpeg4video
  --enable-encoder=h264_videotoolbox,aac,rawvideo
  --enable-muxer=mp4,mov,h264,rawvideo
  --enable-bsf=h264_mp4toannexb
  --enable-filter=aformat,anull,aresample,atrim,color,concat,crop,format,fps,hflip,null,overlay,scale,setpts,setsar,transpose,trim,vflip
)
stamp_file="$binaries_directory/.ffmpeg-$target.stamp"
stamp="$ffmpeg_version $ffmpeg_sha256 ${configure_flags[*]}"
sidecars=(ffmpeg ffprobe)

built() {
  [[ -f "$stamp_file" && "$(cat -- "$stamp_file")" == "$stamp" ]] || return 1
  for program in "${sidecars[@]}"; do
    [[ -x "$binaries_directory/$program-$target" ]] || return 1
  done
}

if [[ "$(uname -s)" != Darwin ]]; then
  printf 'The bundled FFmpeg is built on macOS only.\n' >&2
  exit 1
fi
if built; then
  printf 'FFmpeg %s for %s is already built.\n' "$ffmpeg_version" "$target"
  exit 0
fi
mkdir -p "$cache_directory" "$binaries_directory"
if [[ ! -f "$source_archive" ]] || ! shasum -a 256 -c <<<"$ffmpeg_sha256  $source_archive" >/dev/null 2>&1; then
  partial_archive="$source_archive.partial"
  curl -fL --retry 3 -o "$partial_archive" "https://ffmpeg.org/releases/ffmpeg-$ffmpeg_version.tar.xz"
  if ! shasum -a 256 -c <<<"$ffmpeg_sha256  $partial_archive" >/dev/null; then
    rm -f -- "$partial_archive"
    printf 'The FFmpeg %s source archive does not match the pinned checksum.\n' "$ffmpeg_version" >&2
    exit 1
  fi
  mv -f -- "$partial_archive" "$source_archive"
fi
rm -rf -- "$source_directory" "$object_directory"
tar -xJf "$source_archive" -C "$cache_directory"
mkdir -p "$object_directory"
cd "$object_directory"
export MACOSX_DEPLOYMENT_TARGET="$minimum_macos"
"$source_directory/configure" "${configure_flags[@]}"
if ! grep -q '^#define CONFIG_GPL 0$' config.h || ! grep -q '^#define CONFIG_NONFREE 0$' config.h || ! grep -q '^#define CONFIG_VERSION3 0$' config.h; then
  printf 'The FFmpeg configuration is not LGPL-only.\n' >&2
  exit 1
fi
make -j"$(sysctl -n hw.ncpu)" ffmpeg ffprobe
for program in "${sidecars[@]}"; do
  install -m 755 "$program" "$binaries_directory/$program-$target"
done
mkdir -p "$(dirname -- "$license_file")"
cp -f -- "$source_directory/COPYING.LGPLv2.1" "$license_file"
printf '%s' "$stamp" >"$stamp_file"
for program in "${sidecars[@]}"; do
  printf 'Built: %s\n' "$binaries_directory/$program-$target"
done
