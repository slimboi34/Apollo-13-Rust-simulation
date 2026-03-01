from PIL import Image

try:
    path = "/Users/josh/Projects/lab/matlab/applo11_simulation/apollo11_bevy/assets/"
    img = Image.open(path + "earth_clouds_4k.jpg").convert('L')
    out = Image.new('RGBA', img.size, (255, 255, 255, 255))
    out.putalpha(img)
    out.save(path + "earth_clouds_4k.png")
    print("Cloud map alpha channel injected successfully!")
except Exception as e:
    print(f"Failed: {e}")
