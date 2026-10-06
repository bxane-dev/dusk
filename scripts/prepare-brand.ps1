$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$iconsDir = Join-Path $root "src-tauri\icons"
$installerDir = Join-Path $root "src-tauri\installer"

New-Item -ItemType Directory -Force -Path $iconsDir, $installerDir | Out-Null

function New-RoundedRectPath(
  [float]$x,
  [float]$y,
  [float]$width,
  [float]$height,
  [float]$radius
) {
  $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
  $diameter = $radius * 2
  $path.AddArc($x, $y, $diameter, $diameter, 180, 90)
  $path.AddArc($x + $width - $diameter, $y, $diameter, $diameter, 270, 90)
  $path.AddArc($x + $width - $diameter, $y + $height - $diameter, $diameter, $diameter, 0, 90)
  $path.AddArc($x, $y + $height - $diameter, $diameter, $diameter, 90, 90)
  $path.CloseFigure()
  return $path
}

function New-DuskDPath {
  $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
  $path.FillMode = [System.Drawing.Drawing2D.FillMode]::Alternate

  $path.StartFigure()
  $path.AddLine(39, 31, 123, 31)
  $path.AddBezier(123, 31, 185, 31, 223, 68, 223, 128)
  $path.AddBezier(223, 128, 223, 188, 185, 225, 123, 225)
  $path.AddLine(123, 225, 38, 225)
  $path.AddBezier(38, 225, 50, 211, 54, 197, 54, 175)
  $path.AddLine(54, 175, 54, 80)
  $path.AddBezier(54, 80, 54, 58, 51, 44, 39, 31)
  $path.CloseFigure()

  $path.StartFigure()
  $path.AddEllipse(55, 60, 122, 136)
  $path.CloseFigure()

  return $path
}

function Draw-DuskLogo(
  [System.Drawing.Graphics]$graphics,
  [float]$x,
  [float]$y,
  [float]$size
) {
  $state = $graphics.Save()
  try {
    $scale = $size / 256.0
    $graphics.TranslateTransform($x, $y)
    $graphics.ScaleTransform($scale, $scale)
    $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

    $dPath = New-DuskDPath
    $bounds = [System.Drawing.RectangleF]::new(32, 24, 202, 208)
    $metal = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
      $bounds,
      [System.Drawing.Color]::FromArgb(205, 202, 238),
      [System.Drawing.Color]::FromArgb(91, 28, 214),
      45.0
    )
    $glowWide = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(30, 124, 58, 237), 18)
    $glowMid = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(70, 139, 92, 246), 9)
    $edge = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(205, 181, 146, 255), 2.4)

    try {
      $graphics.DrawPath($glowWide, $dPath)
      $graphics.DrawPath($glowMid, $dPath)
      $graphics.FillPath($metal, $dPath)
      $graphics.DrawPath($edge, $dPath)
    } finally {
      $edge.Dispose()
      $glowMid.Dispose()
      $glowWide.Dispose()
      $metal.Dispose()
      $dPath.Dispose()
    }

    # Violet crescent highlight on the right side of the D.
    $crescent = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $crescent.AddBezier(126, 48, 184, 48, 215, 80, 215, 128)
    $crescent.AddBezier(215, 128, 215, 178, 184, 208, 126, 208)
    $crescent.AddBezier(160, 190, 184, 160, 184, 128)
    $crescent.AddBezier(184, 128, 184, 95, 160, 65, 126, 48)
    $crescent.CloseFigure()
    $violet = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
      [System.Drawing.RectangleF]::new(120, 45, 100, 166),
      [System.Drawing.Color]::FromArgb(245, 238, 255),
      [System.Drawing.Color]::FromArgb(96, 30, 210),
      35.0
    )
    $crescentGlow = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(70, 124, 58, 237), 9)
    try {
      $graphics.DrawPath($crescentGlow, $crescent)
      $graphics.FillPath($violet, $crescent)
    } finally {
      $crescentGlow.Dispose()
      $violet.Dispose()
      $crescent.Dispose()
    }

    # Controller D-pad.
    $padBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
      [System.Drawing.RectangleF]::new(56, 85, 86, 86),
      [System.Drawing.Color]::FromArgb(239, 228, 255),
      [System.Drawing.Color]::FromArgb(109, 40, 217),
      45.0
    )
    $padGlow = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(150, 145, 84, 255), 5)
    $pads = @(
      (New-RoundedRectPath 83 87 22 35 6),
      (New-RoundedRectPath 83 134 22 35 6),
      (New-RoundedRectPath 58 112 35 22 6),
      (New-RoundedRectPath 105 112 35 22 6)
    )
    try {
      foreach ($pad in $pads) {
        $graphics.DrawPath($padGlow, $pad)
        $graphics.FillPath($padBrush, $pad)
      }
      $hub = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(22, 7, 47))
      try {
        $graphics.FillEllipse($hub, 85.5, 114.5, 17, 17)
      } finally {
        $hub.Dispose()
      }
    } finally {
      foreach ($pad in $pads) { $pad.Dispose() }
      $padGlow.Dispose()
      $padBrush.Dispose()
    }
  } finally {
    $graphics.Restore($state)
  }
}

function New-DuskLogoBitmap([int]$size) {
  $bitmap = [System.Drawing.Bitmap]::new(
    $size,
    $size,
    [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
  )
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try {
    $graphics.Clear([System.Drawing.Color]::Transparent)
    Draw-DuskLogo $graphics 0 0 $size
  } finally {
    $graphics.Dispose()
  }
  return $bitmap
}

function Save-Png([string]$path, [int]$size) {
  $bitmap = New-DuskLogoBitmap $size
  try {
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $bitmap.Dispose()
  }
}

function Get-PngBytes([int]$size) {
  $bitmap = New-DuskLogoBitmap $size
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

function New-InstallerBitmap([int]$width, [int]$height) {
  return [System.Drawing.Bitmap]::new(
    $width,
    $height,
    [System.Drawing.Imaging.PixelFormat]::Format24bppRgb
  )
}

function Save-Header([string]$path) {
  $bitmap = New-InstallerBitmap 150 57
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $rect = [System.Drawing.Rectangle]::new(0, 0, 150, 57)
  $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $rect,
    [System.Drawing.Color]::FromArgb(5, 5, 12),
    [System.Drawing.Color]::FromArgb(24, 10, 58),
    0.0
  )
  $accent = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(122, 64, 255), 2)
  try {
    $graphics.FillRectangle($brush, $rect)
    Draw-DuskLogo $graphics 94 2 53
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
  $bitmap = New-InstallerBitmap 164 314
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $rect = [System.Drawing.Rectangle]::new(0, 0, 164, 314)
  $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    $rect,
    [System.Drawing.Color]::FromArgb(4, 4, 10),
    [System.Drawing.Color]::FromArgb(21, 8, 52),
    90.0
  )
  $accent = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(122, 64, 255), 4)
  $accentSoft = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(66, 32, 145), 2)
  try {
    $graphics.FillRectangle($brush, $rect)
    $graphics.DrawLine($accent, 2, 0, 2, 314)
    $graphics.DrawLine($accentSoft, 6, 0, 6, 314)
    Draw-DuskLogo $graphics 10 48 144

    for ($index = 0; $index -lt 36; $index++) {
      $strength = [Math]::Max(25, 90 - ($index * 2))
      $pen = [System.Drawing.Pen]::new(
        [System.Drawing.Color]::FromArgb($strength, 52, 32, 150),
        1
      )
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

Save-Png (Join-Path $iconsDir "32x32.png") 32
Save-Png (Join-Path $iconsDir "128x128.png") 128
Save-Png (Join-Path $iconsDir "128x128@2x.png") 256
Save-Png (Join-Path $iconsDir "icon.png") 256
Write-Ico (Join-Path $iconsDir "icon.ico") @(16, 24, 32, 48, 64, 128, 256)
Save-Header (Join-Path $installerDir "header.bmp")
Save-Sidebar (Join-Path $installerDir "sidebar.bmp")

Write-Host "Dusk branding assets generated from coded vector geometry."
