package main

/*
#include <stdlib.h>
*/
import "C"

import (
	"context"
	"encoding/json"
	"fmt"
	"path/filepath"
	"unsafe"

	"github.com/microsoft/typescript-go/internal/ast"
	"github.com/microsoft/typescript-go/internal/bundled"
	"github.com/microsoft/typescript-go/internal/compiler"
	"github.com/microsoft/typescript-go/internal/diagnostics"
	"github.com/microsoft/typescript-go/internal/locale"
	"github.com/microsoft/typescript-go/internal/scanner"
	"github.com/microsoft/typescript-go/internal/tsoptions"
	"github.com/microsoft/typescript-go/internal/vfs"
	"github.com/microsoft/typescript-go/internal/vfs/osvfs"
)

// Ensure this is not removed by the compiler.
func main() {}

type configHost struct {
	fs  vfs.FS
	cwd string
}

func (h *configHost) FS() vfs.FS                { return h.fs }
func (h *configHost) GetCurrentDirectory() string { return h.cwd }

type diagnosticJSON struct {
	File      string `json:"file,omitempty"`
	Line      int    `json:"line"`
	Column    int    `json:"column"`
	EndLine   int    `json:"end_line"`
	EndColumn int    `json:"end_column"`
	Message   string `json:"message"`
	Code      int32  `json:"code"`
	Category  string `json:"category"`
}

type resultJSON struct {
	Diagnostics []diagnosticJSON `json:"diagnostics"`
	Error       string           `json:"error,omitempty"`
}

func categoryString(c diagnostics.Category) string {
	switch c {
	case diagnostics.CategoryError:
		return "error"
	case diagnostics.CategoryWarning:
		return "warning"
	case diagnostics.CategorySuggestion:
		return "suggestion"
	case diagnostics.CategoryMessage:
		return "message"
	default:
		return "error"
	}
}

//export TsgoCheckProject
func TsgoCheckProject(configPath *C.char) *C.char {
	var result resultJSON
	defer func() {
		if r := recover(); r != nil {
			result = resultJSON{
				Error: fmt.Sprintf("panic: %v", r),
			}
		}
	}()

	goPath := C.GoString(configPath)
	absPath, err := filepath.Abs(goPath)
	if err != nil {
		result.Error = fmt.Sprintf("failed to resolve path: %v", err)
		return marshalResult(result)
	}

	cwd := filepath.Dir(absPath)

	fs := bundled.WrapFS(osvfs.FS())
	host := &configHost{fs: fs, cwd: cwd}

	config, configErrors := tsoptions.GetParsedCommandLineOfConfigFile(absPath, nil, nil, host, nil)
	if config == nil {
		result.Error = "failed to parse tsconfig.json"
		if len(configErrors) > 0 {
			for _, diag := range configErrors {
				result.Diagnostics = append(result.Diagnostics, convertDiagnostic(diag))
			}
		}
		return marshalResult(result)
	}

	compilerHost := compiler.NewCachedFSCompilerHost(cwd, fs, bundled.LibPath(), nil, nil)
	program := compiler.NewProgram(compiler.ProgramOptions{
		Config: config,
		Host:   compilerHost,
	})

	ctx := context.Background()
	allDiagnostics := compiler.GetDiagnosticsOfAnyProgram(
		ctx,
		program,
		nil,
		false,
		program.GetBindDiagnostics,
		program.GetSemanticDiagnostics,
	)
	allDiagnostics = compiler.SortAndDeduplicateDiagnostics(allDiagnostics)

	for _, diag := range allDiagnostics {
		result.Diagnostics = append(result.Diagnostics, convertDiagnostic(diag))
	}

	if result.Diagnostics == nil {
		result.Diagnostics = []diagnosticJSON{}
	}

	return marshalResult(result)
}

func convertDiagnostic(diag *ast.Diagnostic) diagnosticJSON {
	d := diagnosticJSON{
		Message:  diag.Localize(locale.Default),
		Code:     diag.Code(),
		Category: categoryString(diag.Category()),
	}

	if diag.File() != nil {
		d.File = diag.File().FileName()
		line, col := scanner.GetECMALineAndUTF16CharacterOfPosition(diag.File(), diag.Pos())
		d.Line = line
		d.Column = int(col)
		endLine, endCol := scanner.GetECMALineAndUTF16CharacterOfPosition(diag.File(), diag.End())
		d.EndLine = endLine
		d.EndColumn = int(endCol)
	}

	return d
}

func marshalResult(result resultJSON) *C.char {
	data, err := json.Marshal(result)
	if err != nil {
		errJSON := fmt.Sprintf(`{"error":"json marshal failed: %s","diagnostics":[]}`, err.Error())
		return C.CString(errJSON)
	}
	return C.CString(string(data))
}

//export TsgoFree
func TsgoFree(ptr *C.char) {
	C.free(unsafe.Pointer(ptr))
}

//export TsgoVersion
func TsgoVersion() *C.char {
	return C.CString("0.1.0")
}
