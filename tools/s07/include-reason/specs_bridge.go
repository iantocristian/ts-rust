package tsoptions

// Access-only construction of validated matcher metadata. The fixture inputs
// explicitly supply both spellings; this does not stand in for config parsing.
func S07IncludeSpecs(c *TsConfigSourceFile, files, beforeFiles, includes, beforeIncludes []string, isDefault bool) {
	c.configFileSpecs = &configFileSpecs{
		validatedFilesSpec: files, validatedFilesSpecBeforeSubstitution: beforeFiles,
		validatedIncludeSpecs: includes, validatedIncludeSpecsBeforeSubstitution: beforeIncludes,
		isDefaultIncludeSpec: isDefault,
	}
}
