package main

import (
	"crypto/sha256"
	"fmt"
	"io/fs"
	"io/ioutil"
	"os"
	"path"
	"sync"
	"time"
)

type Entry struct {
	filename string
	hash     string
}

type File struct {
	filename   string
	duplicates []string
}

type Duplicates struct {
	mtx        sync.RWMutex
	duplicates map[string]File
}

func (d *Duplicates) Add(entry Entry) {
	d.mtx.Lock()
	defer d.mtx.Unlock()

	f, ok := d.duplicates[entry.hash]
	if !ok {
		d.duplicates[entry.hash] = File{
			filename:   entry.filename,
			duplicates: []string{},
		}
	} else {
		f.duplicates = append(f.duplicates, entry.filename)
		d.duplicates[entry.hash] = f
	}
}

func (d *Duplicates) Get() map[string]File {
	d.mtx.RLock()
	defer d.mtx.RUnlock()

	return d.duplicates
}

func main() {
	channel := make(chan Entry)

	duplicates := Duplicates{
		duplicates: map[string]File{},
	}

	go func() {
		for entry := range channel {
			duplicates.Add(entry)
		}
	}()

	wg := sync.WaitGroup{}

	start := time.Now()
	traverse(os.Args[1], channel, &wg)

	wg.Wait()

	for _, value := range duplicates.Get() {
		if len(value.duplicates) != 0 {
			fmt.Println(value.filename)
			for _, dupe := range value.duplicates {
				fmt.Println("\t", dupe)
			}
		}
	}

	fmt.Println("elapsed time", time.Since(start))
}

func traverse(p string, channel chan Entry, wg *sync.WaitGroup) {
	entries, err := ioutil.ReadDir(p)
	if err != nil {
		fmt.Println(err)
		return
	}

	wg.Add(len(entries))
	for _, entry := range entries {
		e := entry
		fullPath := path.Join(p, entry.Name())
		if entry.IsDir() {
			go func(entry fs.FileInfo) {
				traverse(fullPath, channel, wg)
				wg.Done()
			}(e)
		} else {
			f, err := ioutil.ReadFile(fullPath)
			if err != nil {
				fmt.Println(err)
				continue
			}
			h := sha256.New()
			h.Write(f)
			channel <- Entry{
				filename: fullPath,
				hash:     string(h.Sum(nil)),
			}
			wg.Done() // this could cause deadlocks i think
		}
	}
}
