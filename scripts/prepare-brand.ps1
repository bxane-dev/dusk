$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$sourcePath = Join-Path $root "branding\dusk-logo.base64"
$assetsDir = Join-Path $root "src\assets"
$iconsDir = Join-Path $root "src-tauri\icons"
$installerDir = Join-Path $root "src-tauri\installer"

New-Item -ItemType Directory -Force -Path $assetsDir, $iconsDir, $installerDir | Out-Null

$logoBase64 = (Get-Content $sourcePath -Raw).Trim()
$logoBytes = [Convert]::FromBase64String($logoBase64)
[System.IO.File]::WriteAllBytes((Join-Path $assetsDir "dusk-logo.png"), $logoBytes)

$sourceStream = [System.IO.MemoryStream]::new($logoBytes, $false)
$source = [System.Drawing.Image]::FromStream($sourceStream)

function New-ResizedBitmap([int]$size) {
  $bitmap = [System.Drawing.Bitmap]::new($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try {
    $graphics.Clear([System.Drawing.Color]::Transparent)
    $graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
    $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.DrawImage($source, [System.Drawing.Rectangle]::new(0, 0, $size, $size))
  } finally {
    $graphics.Dispose()
  }
  return $bitmap
}

function Save-Png([string]$path, [int]$size) {
  $bitmap = New-ResizedBitmap $size
  try {
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $bitmap.Dispose()
  }
}

function Get-PngBytes([int]$size) {
  $bitmap = New-ResizedBitmap $size
  $stream = [System.IO.MemoryStream]::new()
  try {
    $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    return $stream.ToArray()
  } finally {
    $stream.Dispose()
    $bitmap.Dispose()
  }
}

function Write-Ico([string]$path, [int[]]$sizes) {
  $images = @()
  foreach ($size in $sizes) {
    $images += [pscustomobject]@{
      Size = $size
      Bytes = Get-PngBytes $size
    }
  }

  $file = [System.IO.File]::Create($path)
  $writer = [System.IO.BinaryWriter]::new($file)
  try {
    $writer.Write([UInt16]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]$images.Count)

    [UInt32]$offset = 6 + (16 * $images.Count)
    foreach ($image in $images) {
      $dimension = if ($image.Size -ge 256) { [byte]0 } else { [byte]$image.Size }
      $writer.Write($dimension)
      $writer.Write($dimension)
      $writer.Write([byte]0)
      $writer.Write([byte]0)
      $writer.Write([UInt16]1)
      $writer.Write([UInt16]32)
      $writer.Write([UInt32]$image.Bytes.Length)
      $writer.Write([UInt32]$offset)
      $offset += [UInt32]$image.Bytes.Length
    }

    foreach ($image in $images) {
      $writer.Write([byte[]]$image.Bytes)
    }
  } finally {
    $writer.Dispose()
    $file.Dispose()
  }
}

function Draw-Logo([System.Drawing.Graphics]$graphics, [int]$x, [int]$y, [int]$size) {
  $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
  $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $graphics.DrawImage($source, [System.Drawing.Rectangle]::new($x, $y, $size, $size))
}

function Save-Header([string]$path) {
  $width = 150
  $height = 57
  $bitmap = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $rect = [System.Drawing.Rectangle]::new(0, 0, $width, $height)
  $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $rect,
    [System.Drawing.Color]::FromArgb(5, 5, 12),
    [System.Drawing.Color]::FromArgb(18, 8, 46),
    0.0
  )
  $accent = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(122, 64, 255), 2)
  try {
    $graphics.FillRectangle($brush, $rect)
    Draw-Logo $graphics 94 2 53
    $graphics.DrawLine($accent, 0, 55, 150, 55)
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp)
  } finally {
    $accent.Dispose()
    $brush.Dispose()
    $graphics.Dispose()
    $bitmap.Dispose()
  }
}

function Save-Sidebar([string]$path) {
  $width = 164
  $height = 314
  $bitmap = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $rect = [System.Drawing.Rectangle]::new(0, 0, $width, $height)
  $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $rect,
    [System.Drawing.Color]::FromArgb(5, 5, 12),
    [System.Drawing.Color]::FromArgb(14, 8, 40),
    90.0
  )
  $accent = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(122, 64, 255), 4)
  $accentSoft = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(66, 32, 145), 2)
  try {
    $graphics.FillRectangle($brush, $rect)
    $graphics.DrawLine($accent, 2, 0, 2, 314)
    $graphics.DrawLine($accentSoft, 6, 0, 6, 314)
    Draw-Logo $graphics 10 48 144

    for ($index = 0; $index -lt 36; $index++) {
      $strength = [Math]::Max(25, 90 - ($index * 2))
      $pen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb($strength, 52, 32, 150), 1)
      try {
        $graphics.DrawLine($pen, 14, 266 + $index, 150, 266 + $index)
      } finally {
        $pen.Dispose()
      }
    }

    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp)
  } finally {
    $accentSoft.Dispose()
    $accent.Dispose()
    $brush.Dispose()
    $graphics.Dispose()
    $bitmap.Dispose()
  }
}

try {
  Save-Png (Join-Path $iconsDir "32x32.png") 32
  Save-Png (Join-Path $iconsDir "128x128.png") 128
  Save-Png (Join-Path $iconsDir "128x128@2x.png") 256
  Save-Png (Join-Path $iconsDir "icon.png") 256
  Write-Ico (Join-Path $iconsDir "icon.ico") @(16, 24, 32, 48, 64, 128, 256)
  Save-Header (Join-Path $installerDir "header.bmp")
  Save-Sidebar (Join-Path $installerDir "sidebar.bmp")
} finally {
  $source.Dispose()
  $sourceStream.Dispose()
}

Write-Host "Dusk branding assets generated."
