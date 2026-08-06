Add-Type -AssemblyName System.Drawing

$bmp = New-Object System.Drawing.Bitmap(32, 32)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(233, 69, 96))
$bmp.Save('J:\Project Prompts\port installer\portable-installer\src-tauri\icons\icon.ico')
$g.Dispose()
$bmp.Dispose()

# Also save as PNG for other sizes
$bmp2 = New-Object System.Drawing.Bitmap(32, 32)
$g2 = [System.Drawing.Graphics]::FromImage($bmp2)
$g2.Clear([System.Drawing.Color]::FromArgb(233, 69, 96))
$bmp2.Save('J:\Project Prompts\port installer\portable-installer\src-tauri\icons\32x32.png')
$g2.Dispose()
$bmp2.Dispose()

$bmp3 = New-Object System.Drawing.Bitmap(128, 128)
$g3 = [System.Drawing.Graphics]::FromImage($bmp3)
$g3.Clear([System.Drawing.Color]::FromArgb(233, 69, 96))
$bmp3.Save('J:\Project Prompts\port installer\portable-installer\src-tauri\icons\128x128.png')
$g3.Dispose()
$bmp3.Dispose()

$bmp4 = New-Object System.Drawing.Bitmap(256, 256)
$g4 = [System.Drawing.Graphics]::FromImage($bmp4)
$g4.Clear([System.Drawing.Color]::FromArgb(233, 69, 96))
$bmp4.Save('J:\Project Prompts\port installer\portable-installer\src-tauri\icons\128x128@2x.png')
$g4.Dispose()
$bmp4.Dispose()

Write-Host "Icons created successfully"
