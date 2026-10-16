Attribute VB_Name = "Demo"
Option Explicit

' A small tour of what cellnux-vba runs today.
' Try it:  cargo run -p cellnux-vba --example run_macro -- crates/vba/examples/demo.bas
' Then:    cargo run --release -p gridcraft -- macro-result.xlsx

Sub BuildReport()
    Dim i As Long, total As Double
    Dim products As Variant
    products = Array("Tea", "Coffee", "Juice", "Water", "Milk")

    Range("A1").Value = "Product"
    Range("B1").Value = "Qty"
    Range("C1").Value = "Price"
    Range("D1").Value = "Total"

    For i = 0 To UBound(products)
        Cells(i + 2, 1).Value = products(i)
        Cells(i + 2, 2).Value = (i + 1) * 3
        Cells(i + 2, 3).Value = 2.5 + i
        Cells(i + 2, 4).Formula = "=B" & (i + 2) & "*C" & (i + 2)
    Next i

    Range("A8").Value = "Grand total"
    Range("D8").Formula = "=SUM(D2:D6)"

    total = Application.WorksheetFunction.Sum(Range("D2:D6"))
    Debug.Print "Grand total computed by the sheet: "; Range("D8").Value
    Debug.Print "Grand total computed by VBA:       "; total
    Debug.Print "Best seller: "; BestSeller()

    If total > 100 Then
        MsgBox "Report ready. Total = " & Format(total, "#,##0.00")
    End If
End Sub

Function BestSeller() As String
    Dim c As Variant, best As Double
    For Each c In Range("D2:D6")
        If c.Value > best Then
            best = c.Value
            BestSeller = c.Offset(0, -3).Value
        End If
    Next c
End Function

Sub TrySandbox()
    ' Macros cannot touch files, processes or the network.
    Shell "calc.exe"
End Sub
