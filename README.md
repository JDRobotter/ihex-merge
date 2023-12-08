# ihex-merge
This tool will merge ihex (intel HEX) binary object files into one.

## usage
  ihex-merge output.hex in1.hex in2.hex in3.hex

### caution
The tool currently do not check for overlapping segments and will keep overlapping segments.
