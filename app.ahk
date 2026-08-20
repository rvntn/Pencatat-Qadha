;@Ahk2Exe-ExeName QadhaTracker
;@Ahk2Exe-SetName Qadha Tracker
;@Ahk2Exe-SetDescription Pencatat Qadha Shalat
;@Ahk2Exe-SetCompanyName Raven
;@Ahk2Exe-SetCopyright Copyright (c) 2026
;@Ahk2Exe-SetVersion 1.0
#Include <JSON>
#NoEnv
#SingleInstance Force
SetWorkingDir %A_ScriptDir%

; --- Konfigurasi Database ---
IniFile := "data_qadha.ini"
DaftarSalat := "Subuh, Dzuhur, Ashar, Maghrib, Isya"
;array pemetaan
MapSalat := {"Subuh": "shubuh", "Dzuhur": "dzuhur", "Ashar": "ashr", "Maghrib": "maghrib", "Isya": "isya"}
IniRead, AreaCode, %IniFile%, Area, kode, 193
url := "https://jadwalsholat.org/adzan/ajax/next/ajax.daily.php?id=" . AreaCode
json := req(url)

; --- Pembuatan UI ---
Gui, +AlwaysOnTop
Gui, Font, s12 Bold, Segoe UI
Gui, Add, Text, Center w200, Pencatat Qadha

Gui, Font, s10 Normal
Loop, Parse, DaftarSalat, `,
{
    SalatName := Trim(A_LoopField)

    JsonKey := MapSalat[SalatName]
    WaktuSalat := json.schedules[JsonKey]
    
    ; Baca data dari INI (Default 0 jika tidak ada)
    IniRead, Count%SalatName%, %IniFile%, Jumlah, %SalatName%, 0
    
    ; Baris Nama Salat
    ;Gui, Add, Text, xm y+10 w70, %SalatName%
    Gui, Add, Text, xm y+10 w30, %WaktuSalat%

    Gui, Add, Text, x+10 w50, %SalatName%
    
    ; Tombol Minus
    Gui, Add, Button, x+10 w30 h20 gKurang vBtnMin%SalatName%, -
    
    ; Display Angka
    Gui, Add, Text, x+10 w40 Center vDisp%SalatName%, % Count%SalatName%
    
    ; Tombol Plus
    Gui, Add, Button, x+10 w30 h20 gTambah vBtnPlus%SalatName%, +
}

Gui, Show,, QadhaTracker
Return

; --- Label Penambah (Tambah) ---
Tambah:
    ; Mengambil nama variabel dari tombol yang ditekan
    CurrentCtrl := A_GuiControl
    StringReplace, SalatID, CurrentCtrl, BtnPlus, , All
    
    Count%SalatID% += 1
    Gosub, UpdateData
return

; --- Label Pengurang (Kurang) ---
Kurang:
    CurrentCtrl := A_GuiControl
    StringReplace, SalatID, CurrentCtrl, BtnMin, , All
    
    if (Count%SalatID% > 0) {
        Count%SalatID% -= 1
        Gosub, UpdateData
    }
return

; --- Update UI dan Simpan ke File ---
UpdateData:
    GuiControl,, Disp%SalatID%, % Count%SalatID%
    IniWrite, % Count%SalatID%, %IniFile%, Jumlah, %SalatID%
return

GuiClose:
    ExitApp

req(URL) {
    whr := ComObjCreate("WinHttp.WinHttpRequest.5.1")
    whr.Open("GET", URL, true)
    whr.Send()
    whr.WaitForResponse()
    parsed := JSON.Load( whr.ResponseText )
    Return parsed
}