package storage

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
)

// SaveData replaces data atomically by using a temp file to perform writes before
// renaming to the target path -- `fsycnc`ing between updates.
//
// Power-loss atomic: Will a reader observe a bad state after a crash?
// Readers-writer atomic: Will a reader observe a bad state with a concurrent writer?
func SaveData(path string, data []byte) error {
	tmp := fmt.Sprintf("%s.tmp.%d", path, rand.Int())
	fp, err := os.OpenFile(tmp, os.O_CREATE|os.O_WRONLY|os.O_EXCL, 0664)
	if err != nil {
		return err
	}

	defer func() {
		fp.Close()
		if err != nil {
			// Discard the temp file if it still exists
			os.Remove(tmp)
		}
	}()

	if _, err = fp.Write(data); err != nil {
		return err
	}
	// Power-loss atomicity
	if err = fp.Sync(); err != nil {
		return err
	}
	if err = fp.Close(); err != nil { 
        return err
    }
	// Readers-write atomicity
	if err = os.Rename(tmp, path); err != nil {
		return err
	}

	// We need to ensure that the rename will not only be readers-writer atomic
	// due to the atomicity of the directory; so, we will synchronize directory
	// operations.
	dir := filepath.Dir(path)
	dirFd, err := os.Open(dir)
	if err != nil {
		return err
	}
	defer dirFd.Close()


	// Power-loss atomicity
	return dirFd.Sync()
}