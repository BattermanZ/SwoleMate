const { createCanvas } = require('canvas');
const fs = require('fs');
const path = require('path');

// Function to create a canvas with the specified size
function createIconCanvas(size) {
  const canvas = createCanvas(size, size);
  const ctx = canvas.getContext('2d');

  // Create gradient background
  const gradient = ctx.createLinearGradient(0, size * 0.2, size, size * 0.8);
  gradient.addColorStop(0, '#652B26');  // Start color
  gradient.addColorStop(1, '#7D3931');  // End color
  
  // Fill background with gradient
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, size, size);

  // Add emoji
  ctx.font = `${Math.floor(size * 0.65)}px "Apple Color Emoji"`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillStyle = '#FFFFFF';

  // Draw emoji slightly above center for better visual balance
  const yOffset = size * -0.05;
  ctx.fillText('💪', size/2, size/2 + yOffset);

  return canvas;
}

// Generate icons for different sizes
const sizes = [192, 512];
const staticDir = path.join(__dirname, '..', 'client', 'static');

// Create the static directory if it doesn't exist
if (!fs.existsSync(staticDir)) {
  fs.mkdirSync(staticDir, { recursive: true });
}

// Generate icons for each size
sizes.forEach(size => {
  try {
    console.log(`\nGenerating ${size}x${size} icon...`);
    const canvas = createIconCanvas(size);
    const buffer = canvas.toBuffer('image/png');
    const filePath = path.join(staticDir, `pwa-${size}.png`);
    fs.writeFileSync(filePath, buffer);
    console.log(`Successfully wrote file: ${filePath}`);
    
    // Verify file was created and has content
    const stats = fs.statSync(filePath);
    console.log(`File size: ${stats.size} bytes`);
  } catch (error) {
    console.error(`Error generating ${size}x${size} icon:`, error);
  }
}); 