# Steganos

Name is the greek word that means "covered" or "hidden"

Steganos is a form of least significant bit [Steganography](https://en.wikipedia.org/wiki/Steganography) that hides text
or images inside other images.

## Features

* Embed text into images
* Embed images into images [Not fully working]
* Read the embedded data out of an image.
* Can write nearly any file type into an image as long as it will fit, see [size constraints](#size-constraints)

## Size Constraints

In order to embed text into images, you'll have to know how much data you can actually fit in your image.

\# of pixels that can be embedded in your image = # of pixels in desired image / 8

In general, every byte of information that's being written into the image will take up 8 bytes in the image. For data
types supported by the program (image and text) you will be notified if the text or image you're trying to embed is too
big. 

![sus](sus.png)