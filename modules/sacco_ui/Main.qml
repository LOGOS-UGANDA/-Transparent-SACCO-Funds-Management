import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    width: 720
    height: 520

    property string statusText: "Loading sacco_core…"
    property string lastResult: ""

    function refreshStatus() {
        try {
            statusText = logos.callModule("sacco_core", "getStatus", [])
        } catch (e) {
            statusText = "sacco_core not loaded — install the core module first"
        }
    }

    function callCore(method, args) {
        try {
            lastResult = logos.callModule("sacco_core", method, args)
        } catch (e) {
            lastResult = "Error: " + e
        }
    }

    Component.onCompleted: refreshStatus()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        Text {
            text: "Transparent SACCO"
            font.pixelSize: 28
            font.bold: true
            Layout.fillWidth: true
        }

        Text {
            text: "Every fee accounted for. Every shilling transparent."
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            opacity: 0.8
        }

        Text {
            text: "Core status: " + statusText
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        RowLayout {
            spacing: 12
            Layout.fillWidth: true

            Button {
                text: "Refresh status"
                onClicked: refreshStatus()
            }

            Button {
                text: "Demo: pay fees"
                onClicked: callCore("payFees", [
                    "institution-account-id",
                    "Demo Payer",
                    "REF-DEMO-001",
                    "1000000"
                ])
            }

            Button {
                text: "Demo: verify"
                onClicked: callCore("verifyPaymentByRef", ["REF-DEMO-001"])
            }
        }

        GroupBox {
            title: "Last core response"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ScrollView {
                anchors.fill: parent
                TextArea {
                    readOnly: true
                    wrapMode: TextEdit.Wrap
                    text: lastResult.length ? lastResult : "No call yet."
                    selectByMouse: true
                }
            }
        }

        Text {
            text: "UX reference screens live in prototype/src/pages/. Wire full flows after LEZ deploy."
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            opacity: 0.65
            font.pixelSize: 12
        }
    }
}
