$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$iconsDir = Join-Path $root "src-tauri\icons"
$installerDir = Join-Path $root "src-tauri\installer"
$uiAssetsDir = Join-Path $root "src\assets"
$uiLogoPath = Join-Path $uiAssetsDir "dusk-logo.png"
$sourcePath = Join-Path $iconsDir "dusk-logo.png"

New-Item -ItemType Directory -Force -Path $iconsDir, $installerDir, $uiAssetsDir | Out-Null

if (-not (Test-Path $sourcePath)) {
  throw "Official Dusk logo source is missing: $sourcePath"
}

function New-Canvas([int]$width, [int]$height, [bool]$alpha = $true) {
  $format = if ($alpha) {
    [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
  } else {
    [System.Drawing.Imaging.PixelFormat]::Format24bppRgb
  }

  return [System.Drawing.Bitmap]::new($width, $height, $format)
}

function Configure-Graphics([System.Drawing.Graphics]$graphics) {
  $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
  $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
}

function Draw-OfficialLogo(
  [System.Drawing.Graphics]$graphics,
  [System.Drawing.Image]$source,
  [float]$x,
  [float]$y,
  [float]$width,
  [float]$height
) {
  Configure-Graphics $graphics
  $graphics.DrawImage($source, $x, $y, $width, $height)
}

function Save-IconPng(
  [System.Drawing.Image]$source,
  [string]$path,
  [int]$size
) {
  $bitmap = New-Canvas $size $size $true
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try {
    $graphics.Clear([System.Drawing.Color]::Transparent)
    Draw-OfficialLogo $graphics $source 0 0 $size $size
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $graphics.Dispose()
    $bitmap.Dispose()
  }
}

function Get-IconPngBytes(
  [System.Drawing.Image]$source,
  [int]$size
) {
  $bitmap = New-Canvas $size $size $true
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $stream = [System.IO.MemoryStream]::new()
  try {
    $graphics.Clear([System.Drawing.Color]::Transparent)
    Draw-OfficialLogo $graphics $source 0 0 $size $size
    $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    return $stream.ToArray()
  } finally {
    $stream.Dispose()
    $graphics.Dispose()
    $bitmap.Dispose()
  }
}

function Write-Ico(
  [System.Drawing.Image]$source,
  [string]$path,
  [int[]]$sizes
) {
  $images = @()
  foreach ($size in $sizes) {
    $images += [pscustomobject]@{
      Size = $size
      Bytes = Get-IconPngBytes $source $size
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

function New-DarkGradientBrush(
  [System.Drawing.Rectangle]$rect,
  [float]$angle = 90.0
) {
  return [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $rect,
    [System.Drawing.Color]::FromArgb(5, 5, 10),
    [System.Drawing.Color]::FromArgb(35, 11, 78),
    $angle
  )
}

function Save-Header(
  [System.Drawing.Image]$source,
  [string]$path
) {
  $bitmap = New-Canvas 150 57 $false
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $rect = [System.Drawing.Rectangle]::new(0, 0, 150, 57)
  $brush = New-DarkGradientBrush $rect 0
  $accent = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(139, 92, 246), 2)
  $accentSoft = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(62, 38, 125), 1)
  try {
    Configure-Graphics $graphics
    $graphics.FillRectangle($brush, $rect)

    # Subtle violet rails to make the stock NSIS frame feel like Dusk.
    $graphics.DrawLine($accentSoft, 0, 3, 150, 3)
    $graphics.DrawLine($accent, 0, 55, 150, 55)

    Draw-OfficialLogo $graphics $source 94 2 53 53
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp)
  } finally {
    $accentSoft.Dispose()
    $accent.Dispose()
    $brush.Dispose()
    $graphics.Dispose()
    $bitmap.Dispose()
  }
}

function Save-Sidebar(
  [System.Drawing.Image]$source,
  [string]$path
) {
  $bitmap = New-Canvas 164 314 $false
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $rect = [System.Drawing.Rectangle]::new(0, 0, 164, 314)
  $brush = New-DarkGradientBrush $rect 90
  $violet = [System.Drawing.Color]::FromArgb(139, 92, 246)
  $accent = [System.Drawing.Pen]::new($violet, 3)
  $accentSoft = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(56, 34, 112), 1)
  try {
    Configure-Graphics $graphics
    $graphics.FillRectangle($brush, $rect)

    # Branded edge rails.
    $graphics.DrawLine($accent, 2, 0, 2, 314)
    $graphics.DrawLine($accentSoft, 7, 0, 7, 314)

    # Glow band behind the emblem.
    for ($i = 0; $i -lt 28; $i++) {
      $alpha = [Math]::Max(10, 70 - ($i * 2))
      $pen = [System.Drawing.Pen]::new(
        [System.Drawing.Color]::FromArgb($alpha, 112, 63, 225),
        1
      )
      try {
        $graphics.DrawLine($pen, 15, 56 + $i, 149, 56 + $i)
        $graphics.DrawLine($pen, 15, 214 - $i, 149, 214 - $i)
      } finally {
        $pen.Dispose()
      }
    }

    Draw-OfficialLogo $graphics $source 16 62 132 132

    # Bottom light sweep.
    for ($i = 0; $i -lt 42; $i++) {
      $alpha = [Math]::Max(8, 80 - ($i * 2))
      $pen = [System.Drawing.Pen]::new(
        [System.Drawing.Color]::FromArgb($alpha, 76, 38, 170),
        1
      )
      try {
        $graphics.DrawLine($pen, 14, 258 + $i, 150, 258 + $i)
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

$source = [System.Drawing.Image]::FromFile($sourcePath)
try {
  Save-IconPng $source $uiLogoPath 512
  Save-IconPng $source (Join-Path $iconsDir "32x32.png") 32
  Save-IconPng $source (Join-Path $iconsDir "128x128.png") 128
  Save-IconPng $source (Join-Path $iconsDir "128x128@2x.png") 256
  Save-IconPng $source (Join-Path $iconsDir "icon.png") 256

  Write-Ico $source (Join-Path $iconsDir "icon.ico") @(16, 24, 32, 48, 64, 128, 256)

  Save-Header $source (Join-Path $installerDir "header.bmp")
  Save-Sidebar $source (Join-Path $installerDir "sidebar.bmp")
} finally {
  $source.Dispose()
}

Write-Host "Dusk branding generated from the full official logo source."

