<!DOCTYPE html>
<html>
<meta charset="UTF-8">
<head>
    <link rel="stylesheet" type="text/css" href="melonbooks.css">
    <link rel="stylesheet" type="text/css" href="/node_modules/material-components-web/dist/material-components-web.css">
    <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet" />
    <script src="/node_modules/material-components-web/dist/material-components-web.js"></script>
    <link rel="icon" type="image/png" href="/images/favicon.png">
    <script src="melonbooks.js"></script>

    <title display=none>Melonbooks</title>
</head>
<body>
    <?php

    $dbpath = __DIR__.DIRECTORY_SEPARATOR."db".DIRECTORY_SEPARATOR."melonbooks.db";
    $pdo = new PDO("sqlite:$dbpath", "", "", array(
                PDO::ATTR_PERSISTENT => true
            ));
    $pdo->setAttribute( PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION );
    $artist_query = $pdo->query("SELECT name from artists");

    /*if ($dbcon->connect_error) {
        echo("Database connection failed " . $dbcon->connect_error);
    }*/

    function update_artist($a) {
        //Todo
    }

    $artists = array();
    function reset_artist_names() {
        GLOBAL $pdo, $artists;
        $artist_query = $pdo->query("SELECT name FROM artists");
        $artist_query->execute();
        $artists = array();
        foreach($artist_query as $entry) {
            $artist = $entry['name'];
            $artists[] = $artist;
        }
    }

    reset_artist_names();

    switch($_POST["submit"]) {
        case 'Addd': {
            if ($_POST["add_artist"]) {
                $artist = trim($_POST["add_artist"]);
                echo "trying to insert " . $artist;
                $insert_artist_stmt = $pdo->prepare("INSERT INTO artists (name) VALUES (?)");
                if ($insert_artist_stmt->execute([$artist])) {
                    update_artist($artist);
                    reset_artist_names();
                    echo "success";
                }
            }
            break;
        }
        case 'Update Artists': {
            echo "Update Artists";
        }
        default: {}
    }

    ?>

    <!--form action='/melonbooks/index.php' method='post'>
        <div class='mdc-text-field mdc-text-field--outlined'>
            <input class='mdc-text-field__input' autocorrect='off' autocomplete='off' name='add_artist' spellcheck='false' maxlength='512'>
            <div class='mdc-notched-outline mdc-notched-outline--upgraded'>
                <div class='mdc-notched-outline__leading'></div>
                <div class='mdc-notched-outline__notch' style=''>
                    <label for='demo-mdc-text-field' class='mdc-floating-label' style=''>Add Artist</label>
                </div>
                <div class='mdc-notched-outline__trailing'></div>
            </div>
        </div>
        <script>
            var nodes = document.querySelectorAll('.mdc-text-field');
            nodes.forEach(function (e, i, p) {
                mdc.textField.MDCTextField.attachTo(e);
            })
        </script>
        <input class='melonbooks-post-button mdc-button mdc-button--outlined' type='submit' name="submit" value='Add'>
        <input class='melonbooks-post-button mdc-button mdc-button--outlined' type='submit' name="submit" value='Update Artists'>
    </form-->
    <div>
        <?php
	echo "<a href='/melonbooks/index.php'>All</a></br>";
        foreach ($artists as $artist) {
            echo "<a href='/melonbooks/index.php?artist=${artist}'>$artist</a></br>";
        }
        ?>
    </div>

    <div>
      <?php
	 if ($_GET['artist']) {
	 $artist = $_GET['artist'];
	 echo "showing products for ${artist}.";
         $products_query = $pdo->prepare("SELECT url, title, artist, site, imgUrl, dateAdded, availability FROM products WHERE availability in ('Available', 'Preorder') AND artist = (:artist) ORDER BY dateAdded DESC, artist ASC", [PDO::ATTR_CURSOR => PDO::CURSOR_FWDONLY]);
         $products_query->execute(['artist' => $artist]);
         } else {
         $products_query = $pdo->prepare("SELECT url, title, artist, site, imgUrl, dateAdded, availability FROM products WHERE availability in ('Available', 'Preorder') ORDER BY dateAdded DESC, artist ASC", [PDO::ATTR_CURSOR => PDO::CURSOR_FWDONLY]);
         $products_query->execute([]);
         }
	 $results = $products_query->fetchAll();
      
         echo "<table><tr>";
         //header
         echo "<th style='width:20%'>Image</th>";
         echo "<th style='width:45%'>Title</th>";
         echo "<th style='width:10%'>Artist</th>";
         echo "<th style='width:5%'>Site</th>";
         echo "<th style='width:10%'>Date Added</th>";
         echo "<th style='width:10%'>Availability</th>";
         echo "</tr>";
         //data  
         foreach ($results as $row)  {
             echo "<tr>";
             echo "<td><img src='{$row[4]}&height=150'></td>";
             echo "<td><a href={$row[0]}>{$row[1]}</a></td>";
             echo "<td>{$row[2]}</td>";
             echo "<td>{$row[3]}</td>";
             echo "<td>{$row[5]}</td>";
             echo "<td>{$row[6]}</td>";
             echo "</tr>";
         } 
	 echo "</table>";
         
      ?>
    </div>
</body>
</html>
